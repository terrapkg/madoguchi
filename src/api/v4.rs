/// This file is part of Madoguchi.
///
/// Madoguchi is free software: you can redistribute it and/or modify it under the terms of
/// the GNU General Public License as published by the Free Software Foundation, either
/// version 3 of the License, or (at your option) any later version.
///
/// Madoguchi is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
/// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
/// See the GNU General Public License for more details.
///
/// You should have received a copy of the GNU General Public License along with Madoguchi.
/// If not, see <https://www.gnu.org/licenses/>.
///
use super::auth::{verify_token, ApiAuth};
use crate::db::{Build, Madoguchi as Mg, Pkg, Repo};
use rocket::futures::StreamExt;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, put, routes, Route};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::{query as q, query_as as qa};
use tracing::{error, info, instrument, trace, warn};

const MAX_LIM: i64 = 100;

pub(crate) fn routes() -> Vec<Route> {
	routes![
		add_pkg,
		del_pkg,
		add_repo,
		del_repo,
		list_pkgs,
		list_repos,
		search_pkgs,
		pkg_info,
		list_builds
	]
}

#[derive(Deserialize)]
struct AddPkgBody {
	ver: String,
	rel: String,
	arch: String,
	dirs: String,
}

#[put("/<repo>/packages/<name>", data = "<package>")]
async fn add_pkg(
	mut db: Connection<Mg>, auth: ApiAuth, repo: String, name: String, package: Json<AddPkgBody>,
) -> Status {
	if !verify_token(&repo, &auth.token) {
		return Status::Forbidden;
	}
	let dirs = package.dirs.strip_suffix('/').unwrap_or(&package.dirs);
	let q = q!(
		"INSERT INTO pkgs(name, repo, ver, rel, arch, dirs) VALUES ($1,$2,$3,$4,$5, $6)",
		name,
		repo,
		package.ver,
		package.rel,
		package.arch,
		dirs
	);
	match q.execute(&mut *db).await {
		Ok(res) => {
			if res.rows_affected() != 1 {
				tracing::error!("Affected more than 1 rows?");
				Status::InternalServerError
			} else {
				Status::Created
			}
		},
		Err(e) => {
			tracing::error!("{e:#?}");
			if let Some(e) = e.as_database_error() {
				if e.code() == Some("23505".into()) {
					// unique_violation
					Status::Conflict
				} else {
					Status::InternalServerError
				}
			} else {
				Status::InternalServerError
			}
		},
	}
}

#[delete("/<repo>/packages/<name>?<ver>&<arch>&<rel>")]
async fn del_pkg(
	mut db: Connection<Mg>, repo: String, name: String, ver: String, arch: String, rel: String,
	auth: ApiAuth,
) -> Status {
	if !verify_token(&repo, &auth.token) {
		return Status::Forbidden;
	}
	let q = q!(
		"DELETE FROM pkgs WHERE name=$1 AND repo=$2 AND ver=$3 AND arch=$4 AND rel=$5",
		name,
		repo,
		ver,
		arch,
		rel,
	);
	if q.execute(&mut *db).await.map_or(false, |r| r.rows_affected() == 1) {
		Status::NoContent
	} else {
		Status::InternalServerError
	}
}

#[derive(Deserialize)]
struct AddRepoBody {
	link: String,
	gh: String,
}

#[put("/repos/<name>", data = "<repo>")]
async fn add_repo(
	mut db: Connection<Mg>, name: String, repo: Json<AddRepoBody>, auth: ApiAuth,
) -> Status {
	if !verify_token(&name, &auth.token) {
		return Status::Forbidden;
	}
	let link = repo.link.strip_suffix('/').unwrap_or(&repo.link);
	let gh = repo.gh.strip_suffix('/').unwrap_or(&repo.gh);
	let q = q!("INSERT INTO repos(name, link, gh) VALUES ($1,$2,$3)", name, link, gh);
	match q.execute(&mut *db).await {
		Ok(res) => {
			if res.rows_affected() != 1 {
				Status::InternalServerError
			} else {
				Status::Created
			}
		},
		Err(e) => {
			if let Some(e) = e.as_database_error() {
				if e.code() == Some("23505".into()) {
					Status::Conflict
				} else {
					Status::InternalServerError
				}
			} else {
				Status::InternalServerError
			}
		},
	}
}

#[delete("/repos/<name>")]
async fn del_repo(mut db: Connection<Mg>, name: String, auth: ApiAuth) -> Status {
	if !verify_token(&name, &auth.token) {
		return Status::Forbidden;
	}
	// the main point is to delete from the `repos` table, so we ignore errors
	// we erase repo refs in pkgs and builds due to the "REFERENCES" (repo is fk)
	let q = q!("DELETE FROM pkgs WHERE repo = $1", name);
	if let Err(e) = q.execute(&mut *db).await {
		error!("DEL REPO {name} pkgs FAIL: {e:#?}");
	}
	let q = q!("DELETE FROM builds WHERE repo = $1", name);
	if let Err(e) = q.execute(&mut *db).await {
		eprintln!("DEL REPO {name} builds FAIL: {e:#?}");
	}
	let q = q!("DELETE FROM repos WHERE name = $1", name);
	q.execute(&mut *db).await.map_or(Status::InternalServerError, |r| {
		if r.rows_affected() == 1 {
			Status::NoContent
		} else if r.rows_affected() == 0 {
			Status::BadRequest
		} else {
			eprintln!("[BUG] Somehow we deleted more than 1 repos?");
			Status::InternalServerError
		}
	})
}

#[get("/repos")]
async fn list_repos(mut db: Connection<Mg>) -> rocket::serde::json::Value {
	let q = qa::<_, Repo>("SELECT * FROM repos").fetch(&mut *db);
	serde_json::json!(q.map(|x| { x.expect("Can't list repos?") }).collect::<Vec<Repo>>().await)
}

#[get("/<repo>/packages?<n>&<order>&<offset>")]
async fn search_pkgs(
	mut db: Connection<Mg>, repo: String, n: Option<i64>, order: Option<String>,
	offset: Option<i64>,
) -> Result<rocket::serde::json::Value, Status> {
	if let Some(n) = n {
		if n > MAX_LIM {
			return Err(Status::NotFound);
		}
	}
	// highly electronegative atoms :3
	let n = n.unwrap_or(MAX_LIM);
	let o = order.unwrap_or("name DESC".into());
	let f = offset.unwrap_or(0);
	let res =
		qa!(Pkg, "SELECT * FROM pkgs WHERE repo=$1 ORDER BY $2 LIMIT $3 OFFSET $4", repo, o, n, f)
			.fetch(&mut *db)
			.map(|x| x.ok())
			.collect::<Vec<Option<Pkg>>>()
			.await;
	if res.iter().any(|x| x.is_none()) {
		Err(Status::NotFound)
	} else {
		Ok(serde_json::json!(res))
	}
}

#[derive(Serialize)]
struct RepologyPkg {
	name: String,
	version: String,
	release: String,
	url: String,
	recipe: String,
	// maintainers: Vec<String>,
	summary: String,
	license: String,
	category: String,
	build: Option<String>,
	arch: String,
}

#[instrument(skip(db))]
#[get("/<repo>/packages-all")]
async fn list_pkgs(
	mut db: Connection<Mg>, repo: String,
) -> Result<rocket::serde::json::Value, Status> {
	let (url, gh) =
		match q!("SELECT link,gh FROM repos WHERE name = $1", repo).fetch_one(&mut *db).await {
			Ok(r) => (r.link, r.gh),
			Err(e) => {
				if e.to_string()
					== "no rows returned by a query that expected to return at least one row"
				{
					return Err(Status::NotFound);
				} else {
					error!("DB err: {}", e.to_string());
					return Err(Status::BadRequest);
				}
			},
		};
	trace!(url, gh, "repo exists");
	let fut = rocket::tokio::spawn(super::repopkgs::parse_primary_xml(url));
	let mut pkgs = vec![];
	let mut res = qa!(Pkg, "SELECT * FROM pkgs WHERE repo=$1", repo).fetch(&mut *db);
	while let Some(item) = res.next().await {
		match item {
			Ok(p) => pkgs.push(p),
			Err(e) => warn!(?e, "while sel pkgs"),
		}
	}
	drop(res); // need to mutably borrow db later
	info!("Found {} packages.", pkgs.len());
	trace!(?pkgs);
	let mut bids = vec![];
	for p in &pkgs {
		bids.push(q!("SELECT id FROM builds WHERE pname=$1 AND pver=$2 AND parch=$3 AND repo=$4 AND succ=true AND prel=$5", p.name, p.ver, p.arch, repo, p.rel).fetch_one(&mut *db).await.ok().map(|x| x.id));
	}
	trace!(?bids);
	let mut infs = match fut.await.unwrap() {
		Some(x) => x,
		None => {
			error!("packages-all basically died");
			return Err(Status::InternalServerError);
		},
	};
	trace!(?infs);
	let mut rpkgs = vec![];
	for (p, b) in pkgs.into_iter().zip(bids) {
		let inf = match infs.remove(&(
			p.name.to_owned(),
			p.ver.to_owned(),
			p.rel.to_owned(),
			p.arch.to_owned(),
		)) {
			Some(i) => i,
			None => continue,
		};
		rpkgs.push(RepologyPkg {
			name: p.name,
			version: p.ver,
			release: p.rel,
			url: format!("{gh}/{}", p.dirs),
			arch: p.arch,
			build: b.map(|x| format!("https://github.com/terrapkg/packages/actions/runs/{x}")),
			category: p.dirs.clone(), // fixme
			license: inf.license,
			// maintainers: vec![], // todo
			recipe: format!("{gh}/{}/anda.hcl", p.dirs),
			summary: inf.summary,
		});
	}
	Ok(serde_json::json!(rpkgs))
}

#[get("/<repo>/packages/<name>")]
async fn pkg_info(
	mut db: Connection<Mg>, repo: String, name: String,
) -> Result<rocket::serde::json::Value, Status> {
	let res = qa!(Pkg, "SELECT * FROM pkgs WHERE repo=$1 AND name=$2", repo, name)
		.fetch_one(&mut *db)
		.await;
	if let Ok(res) = res {
		Ok(serde_json::json!(res))
	} else {
		Err(Status::NotFound)
	}
}

#[get("/<repo>/builds/<pkg>")]
async fn list_builds(
	mut db: Connection<Mg>, repo: String, pkg: String,
) -> Result<rocket::serde::json::Value, Status> {
	let res = qa!(Build, "SELECT * FROM builds WHERE repo=$1 AND pname=$2", repo, pkg);
	if let Ok(builds) = res.fetch_all(&mut *db).await {
		Ok(serde_json::json!(builds))
	} else {
		Err(Status::NotFound)
	}
}

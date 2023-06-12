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
use super::auth::ApiAuth;
use crate::db::{Build, Madoguchi as Mg, Pkg, Repo};
use rocket::futures::StreamExt;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, put, routes, Route};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::{query as q, query_as as qa};
use tracing::error;

const MAX_LIM: i64 = 100;

pub(crate) fn routes() -> Vec<Route> {
	routes![
		add_pkg,
		del_pkg,
		add_repo,
		del_repo,
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

#[put("/<repo>/packages/<name>", data = "<p>")]
async fn add_pkg(
	mut db: Connection<Mg>, _auth: ApiAuth, repo: String, name: String, p: Json<AddPkgBody>,
) -> Status {
	let dirs = p.dirs.strip_suffix('/').unwrap_or(&p.dirs);
	let q = q!(
		"INSERT INTO pkgs(name,repo,ver,rel,arch,dirs) VALUES ($1,$2,$3,$4,$5,$6)",
		name,
		repo,
		p.ver,
		p.rel,
		p.arch,
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
					let q = q!(
						"UPDATE pkgs SET (ver,rel,dirs)=($3,$4,$6) WHERE (name,repo,arch)=($1,$2,$5)",
						name, repo, p.ver, p.rel, p.arch, dirs
					);
					if q.execute(&mut *db).await.is_ok() {
						return Status::NoContent;
					}
				}
			}
			Status::InternalServerError
		},
	}
}

#[delete("/<repo>/packages/<name>?<ver>&<arch>&<rel>")]
async fn del_pkg(
	mut db: Connection<Mg>, repo: String, name: String, ver: String, arch: String, rel: String,
	_auth: ApiAuth,
) -> Status {
	let q = q!(
		"DELETE FROM pkgs WHERE name=$1 AND repo=$2 AND ver=$3 AND arch=$4 AND rel=$5",
		name,
		repo,
		ver,
		arch,
		rel
	);
	if q.execute(&mut *db).await.is_ok() {
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
	mut db: Connection<Mg>, name: String, repo: Json<AddRepoBody>, _auth: ApiAuth,
) -> Status {
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
					let q =
						q!("UPDATE repos SET (link, gh) = ($2,$3) WHERE name=$1", name, link, gh);
					if q.execute(&mut *db).await.is_ok() {
						return Status::NoContent;
					}
				}
			}
			Status::InternalServerError
		},
	}
}

#[delete("/repos/<name>")]
async fn del_repo(mut db: Connection<Mg>, name: String, _auth: ApiAuth) -> Status {
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

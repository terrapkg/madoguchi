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
use crate::db::{Madoguchi as Mg, Pkg, Repo};
use rocket::futures::StreamExt;
use rocket::http::Status;
use rocket::response::{status, stream::TextStream};
use rocket::serde::json::Json;
use rocket::{delete, get, put, routes, Response, Route};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::query_as as qa;

const MAX_LIM: i64 = 100;

pub(crate) fn routes() -> Vec<Route> {
	routes![add_pkg, del_pkg, add_repo, del_repo, list_pkgs, list_repos, pkg_info]
}

#[derive(Deserialize)]
struct AddPkgBody {
	id: Option<String>,
	verl: String,
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
	let dirs = package.dirs.strip_suffix("/").unwrap_or(&package.dirs);
	let q = sqlx::query!(
		"INSERT INTO pkgs(name, repo, verl, arch, dirs, build) VALUES ($1,$2,$3,$4,$5,$6)",
		name,
		repo,
		package.verl,
		package.arch,
		dirs,
		package.id
	);
	match q.execute(&mut *db).await {
		Ok(res) => {
			if res.rows_affected() != 1 {
				eprintln!("Affected more than 1 rows?");
				Status::InternalServerError
			} else {
				Status::NoContent
			}
		},
		Err(e) => {
			eprintln!("{e:#?}");
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

#[delete("/<repo>/packages/<name>?<verl>&<arch>")]
async fn del_pkg(
	mut db: Connection<Mg>, repo: String, name: String, verl: String, arch: String, auth: ApiAuth,
) -> Status {
	if !verify_token(&repo, &auth.token) {
		return Status::Forbidden;
	}
	let q = sqlx::query!(
		"DELETE FROM pkgs WHERE name=$1 AND repo=$2 AND verl=$3 AND arch=$4",
		name,
		repo,
		verl,
		arch
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
	let link = repo.link.strip_suffix("/").unwrap_or(&repo.link);
	let gh = repo.gh.strip_suffix("/").unwrap_or(&repo.gh);
	let q = sqlx::query!("INSERT INTO repos(name, link, gh) VALUES ($1,$2,$3)", name, link, gh);
	match q.execute(&mut *db).await {
		Ok(res) => {
			if res.rows_affected() != 1 {
				Status::InternalServerError
			} else {
				Status::NoContent
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
	let q = sqlx::query!("DELETE FROM pkgs WHERE repo = $1", name);
	if let Err(e) = q.execute(&mut *db).await {
		eprintln!("DEL REPO {name} pkgs FAIL: {e:#?}");
	}
	let q = sqlx::query!("DELETE FROM builds WHERE repo = $1", name);
	if let Err(e) = q.execute(&mut *db).await {
		eprintln!("DEL REPO {name} builds FAIL: {e:#?}");
	}
	let q = sqlx::query!("DELETE FROM repos WHERE name = $1", name);
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
async fn list_pkgs(
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
	url: String,
	recipe: String,
	maintainers: Vec<String>,
	summary: String,
	license: Option<String>,
	category: String,
	rpms: Vec<String>,
	build: Option<String>,
	arch: String,
}

// TODO(lleyton): I- what
#[get("/<repo>/packages")]
async fn pkg_info(mut db: Connection<Mg>, repo: String) -> TextStream![String] {
	TextStream! {
		let r = match qa!(Repo, "SELECT * FROM repos WHERE name = $1", repo).fetch_one(&mut *db).await {
			Ok(r) => r,
			Err(e) => {
				if e.to_string() == "no rows returned by a query that expected to return at least one row" {
					yield serde_json::json!({
						"status": "404",
						"message": "repo not found"
					}).to_string();
				} else {
					yield serde_json::json!({
						"status": "400",
						"message": e.to_string(),
					}).to_string();
				}
				return;
			}
		};
		let mut res = qa!(Pkg, "SELECT * FROM pkgs WHERE repo=$1", repo).fetch(&mut *db);
		yield "[".into();
		let first = true;
		while let Some(item) = res.next().await {
			if !first {
				yield ",".into();
			}
			if let Ok(pkg) = item {
			yield serde_json::json!( RepologyPkg {
				name: pkg.name,
				version: pkg.verl,
				url: format!("{}/{}", r.gh, pkg.dirs.clone()),
				arch: pkg.arch,
				build: pkg.build.map(|b| format!("https://github.com/terrapkg/packages/actions/runs/{}", b)),
				category: pkg.dirs.clone(),
				license: None, // todo
				maintainers: vec![], // todo
				recipe: format!("{}/{}/anda.hcl", r.gh, pkg.dirs.clone()),
				rpms: vec![], // todo
				summary: "".into() // todo
			}).to_string();
		}
		yield "]".into();
		}
	}
}

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
use crate::db::Madoguchi as Mg;
use rocket::http::Status;
use rocket::{get, routes, Route};
use rocket_db_pools::Connection;

pub(crate) fn routes() -> Vec<Route> {
	routes![add_pkg, del_pkg, add_repo, del_repo]
}

#[get("/<repo>/add/p/<name>?<verl>&<arch>&<dirs>&<runid>")]
async fn add_pkg(
	mut db: Connection<Mg>, repo: String, name: String, verl: String, arch: String, dirs: String,
	runid: Option<String>, auth: ApiAuth,
) -> Status {
	if !verify_token(&name, &auth.token) {
		return Status::Forbidden;
	}
	let runid: Option<i32> = if let Some(runid) = runid {
		match runid.parse() {
			Ok(runid) => Some(runid),
			Err(_) => return Status::BadRequest, // <== return disallows match {}
		}
	} else {
		None
	};
	let dirs = dirs.strip_suffix("/").unwrap_or(&dirs);
	let q = sqlx::query!(
		"INSERT INTO pkgs(name, repo, verl, arch, dirs, build) VALUES ($1,$2,$3,$4,$5,$6)",
		name,
		repo,
		verl,
		arch,
		dirs,
		runid
	);
	match q.execute(&mut *db).await {
		Ok(res) => {
			if res.rows_affected() != 1 {
				Status::InternalServerError
			} else {
				Status::Ok
			}
		},
		Err(e) => {
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

#[get("/<repo>/del/p/<name>?<verl>&<arch>")]
async fn del_pkg(
	mut db: Connection<Mg>, repo: String, name: String, verl: String, arch: String, auth: ApiAuth,
) -> Status {
	if !verify_token(&name, &auth.token) {
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
		Status::Ok
	} else {
		Status::InternalServerError
	}
}

#[get("/repos/add/<name>?<link>&<gh>")]
async fn add_repo(
	mut db: Connection<Mg>, name: String, link: String, gh: String, auth: ApiAuth,
) -> Status {
	if !verify_token(&name, &auth.token) {
		return Status::Forbidden;
	}
	let link = link.strip_suffix("/").unwrap_or(&link);
	let gh = gh.strip_suffix("/").unwrap_or(&gh);
	let q = sqlx::query!("INSERT INTO repos(name, link, gh) VALUES ($1,$2,$3)", name, link, gh);
	match q.execute(&mut *db).await {
		Ok(res) => {
			if res.rows_affected() != 1 {
				Status::InternalServerError
			} else {
				Status::Ok
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

#[get("/repos/del/<name>")]
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
			Status::Ok
		} else if r.rows_affected() == 0 {
			Status::BadRequest
		} else {
			eprintln!("[BUG] Somehow we deleted more than 1 repos?");
			Status::InternalServerError
		}
	})
}

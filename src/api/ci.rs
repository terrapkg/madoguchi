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
use crate::db::{Build, Madoguchi as Mg, Repo};
use rocket::http::Status;
use rocket::{get, routes, Route};
use rocket_db_pools::Connection;
use sqlx::types::chrono;

use super::auth::{verify_token, ApiAuth, REQ};

const POLL: u64 = 30; // poll every n seconds

pub(crate) fn routes() -> Vec<Route> {
	routes![add_build]
}

#[get("/<repo>/add/builds/<name>?<verl>&<arch>&<dirs>&<id>")]
async fn add_build(
	mut db: Connection<Mg>, repo: String, name: String, verl: String, arch: String, dirs: String,
	id: String, auth: ApiAuth,
) -> Status {
	if !verify_token(&id, &auth.token) {
		return Status::Forbidden;
	}
	let d = dirs.trim_matches('/');
	let ep = chrono::Utc::now().naive_utc();
	let q = sqlx::query_as!(
		Build,
		"INSERT INTO builds(pname,pverl,parch,id,repo,epoch) VALUES ($1,$2,$3,$4,$5,$6) RETURNING *",
		name,
		verl,
		arch,
		id,
		repo,
		ep
	);
	let build = match q.fetch_one(&mut *db).await {
		Ok(r) => r,
		Err(e) => {
			eprintln!("{e:?}");
			return Status::InternalServerError;
		},
	};
	let hdl = rocket::tokio::runtime::Handle::current();
	hdl.spawn(track_build(db, build, d.to_string())); // to_string because can't move
	Status::Ok
}

async fn track_build(mut db: Connection<Mg>, build: Build, dirs: String) {
	let repo = sqlx::query_as!(Repo, "SELECT * FROM repos WHERE name = $1", build.repo);
	let repo = repo.fetch_one(&mut *db).await.expect("REPO DOESN'T EXIST???");
	let mut url = repo.gh.replace("github.com", "api.github.com/repos");
	url.push_str("/actions/runs/");
	url.push_str(&build.id);
	loop {
		let resp = REQ.get(&url).send().await.expect("Failed to send reqs to track build");
		let obj: serde_json::Value = resp.json().await.expect("Failed to decode json");
		if obj["status"] == "completed" {
			if obj["conclusion"] == "success" {
				add_pkg(db, build, &dirs).await;
			}
			break;
		}
		rocket::tokio::time::sleep(std::time::Duration::from_secs(POLL)).await;
	}
}

async fn add_pkg(mut db: Connection<Mg>, build: Build, dirs: &str) {
	let q = sqlx::query!(
		"INSERT INTO pkgs(name, repo, verl, arch, dirs, build) VALUES ($1,$2,$3,$4,$5,$6)",
		build.pname,
		build.repo,
		build.pverl,
		build.parch,
		dirs,
		build.id
	);
	assert_eq!(q.execute(&mut *db).await.expect("Failed to insert new pkg").rows_affected(), 1);
}

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
use crate::db::{Build, Madoguchi as Mg};
use rocket::http::Status;
use rocket::{get, routes, Route};
use rocket_db_pools::Connection;
use sqlx::types::chrono;

pub(crate) fn routes() -> Vec<Route> {
	routes![add_build]
}

#[get("/<repo>/add/builds/<name>?<verl>&<arch>&<dirs>&<id>")]
async fn add_build(
	mut db: Connection<Mg>, repo: String, name: String, verl: String, arch: String,
	dirs: Option<String>, id: String, auth: ApiAuth,
) -> Status {
	if !verify_token(&repo, &auth.token) {
		return Status::Forbidden;
	}
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
	if let Some(d) = dirs {
		let d = d.trim_matches('/');
		let q = sqlx::query!(
			"INSERT INTO pkgs(name, repo, verl, arch, dirs, build) VALUES ($1,$2,$3,$4,$5,$6)",
			name,
			repo,
			verl,
			arch,
			d,
			build.id
		);
		assert_eq!(q.execute(&mut *db).await.expect("Failed to insert new pkg").rows_affected(), 1);
	}
	Status::Ok
}

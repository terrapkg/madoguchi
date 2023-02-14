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
use crate::db::{Build, Madoguchi as Mg};
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::{get, routes, Route};
use rocket_db_pools::Connection;
use sqlx::types::chrono;

pub(crate) fn routes() -> Vec<Route> {
	routes![]
}

#[get("/<repo>/add/b/<name>?<v>&<a>&<d>&<l>")]
async fn add_bulid(
	mut db: Connection<Mg>, repo: String, name: String, v: String, a: String, d: String, l: String,
) -> Status {
	// track the build and see if it passes
	let ep = chrono::Utc::now().naive_utc();
	let q = sqlx::query_as!(
		Build,
		"INSERT INTO builds(pname,pverl,parch,link,repo,epoch) VALUES ($1,$2,$3,$4,$5,$6) RETURNING *",
		name,
		v,
		a,
		l,
		repo,
		ep
	);
	let builds = match q.fetch_one(&mut *db).await {
		Ok(r) => r,
		Err(e) => {
			eprintln!("{e:?}");
			return Status::InternalServerError;
		},
	};
	Status::Ok
}

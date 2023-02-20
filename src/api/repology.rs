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
use crate::db::Madoguchi as Mg;
use rocket::response::Redirect;
use rocket::{get, routes, Route};
use rocket_db_pools::Connection;

pub(crate) fn routes() -> Vec<Route> {
	routes![redirect_pkg, redirect_andahcl]
}

#[get("/<repo>/packages/<name>")]
async fn redirect_pkg(mut db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	let dirs = sqlx::query!(
		"SELECT dirs FROM pkgs WHERE name = $1 AND repo = $2 ORDER BY verl DESC",
		name,
		repo
	);
	let dirs = dirs.fetch_one(&mut *db).await.ok()?.dirs;
	let link = sqlx::query!("SELECT link FROM repos WHERE name = $1", name);
	let link = link.fetch_one(&mut *db).await.ok()?.link;
	Some(Redirect::to(format!("{link}/{dirs}")))
}
#[get("/<repo>/packages/<name>/recipe")]
async fn redirect_andahcl(mut db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	let dirs = sqlx::query!(
		"SELECT dirs FROM pkgs WHERE name = $1 AND repo = $2 ORDER BY verl DESC",
		name,
		repo
	);
	let dirs = dirs.fetch_one(&mut *db).await.ok()?.dirs;
	let link = sqlx::query!("SELECT link FROM repos WHERE name = $1", name);
	let link = link.fetch_one(&mut *db).await.ok()?.link;
	Some(Redirect::to(format!("{link}/{dirs}/anda.hcl")))
}

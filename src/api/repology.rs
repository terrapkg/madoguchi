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
	routes![redirect_pkg, redirect_andahcl, redirect_andaspec, redirect_andaspecraw]
}

#[get("/<repo>/packages/<name>")]
async fn redirect_pkg(mut db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	let dirs = sqlx::query!(
		"SELECT dirs FROM pkgs WHERE name = $1 AND repo = $2 ORDER BY ver DESC",
		name,
		repo
	);
	let dirs = dirs.fetch_one(&mut *db).await.ok()?.dirs;
	let link = sqlx::query!("SELECT gh FROM repos WHERE name = $1", repo);
	let link = link.fetch_one(&mut *db).await.ok()?.gh;
	Some(Redirect::to(format!("{link}/{dirs}")))
}
#[get("/<repo>/packages/<name>/hcl")]
async fn redirect_andahcl(mut db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	let dirs = sqlx::query!(
		"SELECT dirs FROM pkgs WHERE name = $1 AND repo = $2 ORDER BY ver DESC",
		name,
		repo
	);
	let dirs = dirs.fetch_one(&mut *db).await.ok()?.dirs;
	let link = sqlx::query!("SELECT gh FROM repos WHERE name = $1", repo);
	let link = link.fetch_one(&mut *db).await.ok()?.gh;
	Some(Redirect::to(format!("{link}/{dirs}/anda.hcl")))
}
#[get("/<repo>/packages/<name>/spec")]
async fn redirect_andaspec(mut db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	let dirs = sqlx::query!(
		"SELECT dirs FROM pkgs WHERE name = $1 AND repo = $2 ORDER BY ver DESC",
		name,
		repo
	);
	let dirs = dirs.fetch_one(&mut *db).await.ok()?.dirs;
	let link = sqlx::query!("SELECT gh FROM repos WHERE name = $1", repo);
	let link = link.fetch_one(&mut *db).await.ok()?.gh;
	let rawurl = link.replace("github.com", "raw.githubusercontent.com").replace("/tree/", "/");
	let hcl = match reqwest::get(format!("{rawurl}/{dirs}/anda.hcl")).await {
		Ok(r) => anda_config::load_from_string(&r.text().await.ok()?).ok()?,
		Err(err) => {
			tracing::error!(?err, ?rawurl, ?dirs, ?repo, ?name, "No hcl found.");
			return None;
		},
	};
	let (_, p) = hcl.project.into_iter().next()?;
	Some(Redirect::to(format!("{link}/{dirs}/{}", p.rpm?.spec.display())))
}
#[get("/<repo>/packages/<name>/spec/raw")]
async fn redirect_andaspecraw(
	mut db: Connection<Mg>, repo: String, name: String,
) -> Option<Redirect> {
	let dirs = sqlx::query!(
		"SELECT dirs FROM pkgs WHERE name = $1 AND repo = $2 ORDER BY ver DESC",
		name,
		repo
	);
	let dirs = dirs.fetch_one(&mut *db).await.ok()?.dirs;
	let link = sqlx::query!("SELECT gh FROM repos WHERE name = $1", repo);
	let link = link.fetch_one(&mut *db).await.ok()?.gh;
	let rawurl = link.replace("github.com", "raw.githubusercontent.com").replace("/tree/", "/");
	let hcl = match reqwest::get(format!("{rawurl}/{dirs}/anda.hcl")).await {
		Ok(r) => anda_config::load_from_string(&r.text().await.ok()?).ok()?,
		Err(err) => {
			tracing::error!(?err, ?rawurl, ?dirs, ?repo, ?name, "No hcl found.");
			return None;
		},
	};
	let (_, p) = hcl.project.into_iter().next()?;
	Some(Redirect::to(format!("{rawurl}/{dirs}/{}", p.rpm?.spec.display())))
}

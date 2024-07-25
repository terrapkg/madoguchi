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

async fn rootdir(mut db: Connection<Mg>, repo: String, name: String) -> Option<String> {
	let dirs = sqlx::query!(
		"SELECT dirs FROM pkgs WHERE name = $1 AND repo = $2 ORDER BY ver DESC",
		name,
		repo
	);
	let dirs = dirs.fetch_one(&mut **db).await.ok()?.dirs;
	let link = sqlx::query!("SELECT gh FROM repos WHERE name = $1", repo);
	let link = link.fetch_one(&mut **db).await.ok()?.gh;
	let commit = sqlx::query!(
		"SELECT commit FROM builds WHERE pname=$1 AND repo=$2 AND succ ORDER BY epoch DESC LIMIT 1",
		name,
		repo
	);
	let mut s = format!("{link}/{dirs}");
	if let Some(commit) = commit.fetch_one(&mut **db).await.ok().and_then(|x| x.commit) {
		s = format!(
			"https://github.com/terrapkg/packages/tree/{commit}{}",
			s.trim_start_matches("https://github.com/terrapkg/packages/tree/")
				.trim_start_matches(|ch| ch != '/')
		);
	}
	tracing::trace!(s);
	Some(s)
}

#[get("/<repo>/packages/<name>")]
async fn redirect_pkg(db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	Some(Redirect::to(rootdir(db, repo, name).await?))
}
#[get("/<repo>/packages/<name>/hcl")]
async fn redirect_andahcl(db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	Some(Redirect::to(format!("{}/anda.hcl", rootdir(db, repo, name).await?)))
}
#[get("/<repo>/packages/<name>/spec")]
async fn redirect_andaspec(db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	let rootdir = rootdir(db, repo, name).await?;
	let rawurl = rootdir.replace("github.com", "raw.githubusercontent.com").replace("/tree/", "/");
	let hcl = match reqwest::get(format!("{rawurl}/anda.hcl")).await {
		Ok(r) => anda_config::load_from_string(&r.text().await.ok()?).ok()?,
		Err(err) => {
			tracing::error!(?err, ?rawurl, "No hcl found.");
			return None;
		},
	};
	let (_, p) = hcl.project.into_iter().next()?;
	Some(Redirect::to(format!("{rootdir}/{}", p.rpm?.spec.display())))
}
#[get("/<repo>/packages/<name>/spec/raw")]
async fn redirect_andaspecraw(db: Connection<Mg>, repo: String, name: String) -> Option<Redirect> {
	let rootdir = rootdir(db, repo, name).await?;
	let rawurl = rootdir.replace("github.com", "raw.githubusercontent.com").replace("/tree/", "/");
	let hcl = match reqwest::get(format!("{rawurl}/anda.hcl")).await {
		Ok(r) => anda_config::load_from_string(&r.text().await.ok()?).ok()?,
		Err(err) => {
			tracing::error!(?err, ?rawurl, "No hcl found.");
			return None;
		},
	};
	let (_, p) = hcl.project.into_iter().next()?;
	Some(Redirect::to(format!("{rawurl}/{}", p.rpm?.spec.display())))
}

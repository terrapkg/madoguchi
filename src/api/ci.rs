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
use crate::db::Madoguchi as Mg;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{put, routes, Route};
use rocket_db_pools::Connection;
use serde::Deserialize;
use sqlx::types::chrono;
use std::sync::LazyLock;

static WEBHOOKER: LazyLock<webhook::client::WebhookClient> = LazyLock::new(|| {
	webhook::client::WebhookClient::new(&std::env::var("DISCORD_WEBHOOK").unwrap())
});

pub fn routes() -> Vec<Route> {
	routes![add_build]
}

#[derive(Deserialize, Debug)]
struct AddBuildBody {
	id: String,
	ver: String,
	rel: String,
	arch: String,
	dirs: String,
	succ: bool,
}

#[put("/<repo>/builds/<name>", data = "<build_body>")]
async fn add_build(
	mut db: Connection<Mg>, repo: String, name: String, build_body: Json<AddBuildBody>,
	_auth: ApiAuth,
) -> Status {
	if !build_body.succ {
		return add_failed_build(db, repo, build_body).await;
	}
	if sqlx::query!(
		"SELECT name FROM pkgs WHERE name=$1 AND repo=$2 AND arch=$3",
		name,
		repo,
		build_body.arch
	)
	.fetch_one(&mut **db)
	.await
	.is_err()
	{
		if let Err(err) = sqlx::query!(
			"INSERT INTO pkgs(name, repo, ver, rel, arch, dirs) VALUES ($1,$2,$3,$4,$5,$6)",
			name,
			repo,
			build_body.ver,
			build_body.rel,
			build_body.arch,
			build_body.dirs.trim_matches('/'),
		)
		.execute(&mut **db)
		.await
		{
			tracing::error!(?build_body, repo, name, ?err, "Cannot add pkgs");
			return Status::InternalServerError;
		}
	} else if build_body.succ {
		// don't want to update if it doesn't even build
		if let Err(err) = sqlx::query!(
			"UPDATE pkgs SET ver=$1,rel=$2,dirs=$3 WHERE name=$4 AND repo=$5 AND arch=$6",
			build_body.ver,
			build_body.rel,
			build_body.dirs.trim_matches('/'),
			name,
			repo,
			build_body.arch,
		)
		.execute(&mut **db)
		.await
		{
			tracing::error!(?build_body, repo, name, ?err, "Cannot update pkgs");
			return Status::InternalServerError;
		}
	}
	let ep = chrono::Utc::now().naive_utc();
	let q = sqlx::query_as!(
		Build,
		"INSERT INTO builds(pname,pver,prel,parch,id,repo,epoch,succ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
		name,
		build_body.ver,
		build_body.rel,
		build_body.arch,
		build_body.id,
		repo,
		ep,
		build_body.succ,
	);
	match q.execute(&mut **db).await {
		Ok(_) => Status::Created,
		Err(e) => {
			eprintln!("{e:?}");
			Status::InternalServerError
		},
	}
}

async fn add_failed_build(mut db: Connection<Mg>, r: String, b: Json<AddBuildBody>) -> Status {
	let q = sqlx::query!("SELECT name FROM pkgs WHERE (dirs,repo)=($1,$2)", b.dirs, &r);
	let names: Vec<String> = match q.fetch_all(&mut **db).await {
		Ok(r) => r.into_iter().map(|r| r.name).collect(),
		Err(_) => return Status::NotFound,
	};
	let ep = chrono::Utc::now().naive_utc();
	for name in names {
		let q = sqlx::query!("INSERT INTO builds(pname,pver,prel,parch,id,repo,epoch,succ) VALUES ($1,$2,$3,$4,$5,$6,$7,false)",name,b.ver,b.rel,b.arch,b.id,r,ep);
		if let Err(e) = q.execute(&mut **db).await {
			tracing::error!(err=?e, "Ignoring error during insertion of failed build");
		}
	}
	let (arch, dir, bid) = (&b.arch, &b.dirs, &b.id);

	send_webhook(format!(
		"
<:incorrect:1176633989864362094> Build Failing on **{r}**: **{dir}** (*{arch}*)
⇒ [gha](https://github.com/terrapkg/packages/actions/runs/{bid}/)",
	))
	.await;

	Status::NoContent
}

async fn send_webhook(s: String) {
	let msg = WEBHOOKER.send(|msg| {
		msg.username("Terra Webhook (mg)")
			.avatar_url("https://avatars.githubusercontent.com/u/114906088")
			.content(&s)
	});
	_ = msg.await;
}

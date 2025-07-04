// Madoguchi -- web server for Terra.
//
// This file is part of Madoguchi.
//
// Madoguchi is free software: you can redistribute it and/or modify it under the terms of
// the GNU General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// Madoguchi is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with Madoguchi.
// If not, see <https://www.gnu.org/licenses/>.
//
mod api;
mod db;
use rocket::{fairing, get, launch, response, routes, Build, Rocket};
use rocket_db_pools::Database;
use tracing::{error, info};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

#[get("/")]
fn index() -> response::Redirect {
	response::Redirect::to("https://terra.fyralabs.com/")
}

#[get("/health")]
const fn health() -> &'static str {
	env!("CARGO_PKG_VERSION")
}

fn chks() {
	assert!(std::env::var("JWT_KEY").is_ok(), "JWT_KEY cannot be empty.");
	assert!(std::env::var("DISCORD_WEBHOOK").is_ok(), "DISCORD_WEBHOOK cannot be empty.");
}

async fn migrate(rocket: Rocket<Build>) -> fairing::Result {
	match db::Madoguchi::fetch(&rocket) {
		Some(db) => match rocket_db_pools::sqlx::migrate!().run(&**db).await {
			Ok(()) => Ok(rocket),
			Err(e) => {
				error!("Fail to init db: {e}");
				Err(rocket)
			},
		},
		None => Err(rocket),
	}
}

#[launch]
fn rocket() -> _ {
	if let Err(e) = dotenv::dotenv() {
		tracing::warn!("Ignoring .env: {e}");
	}
	Registry::default().with(EnvFilter::from_default_env()).with(tracing_logfmt::layer()).init();
	chks();
	info!("Launching rocket 🚀");
	rocket::build()
		.attach(db::Madoguchi::init())
		.attach(rocket::fairing::AdHoc::try_on_ignite("Migrations", migrate))
		.mount("/", routes![index, health])
		.mount("/redirect", api::repology::routes())
		.mount("/ci", api::ci::routes())
		.mount("/ci5", api::ci5::routes())
		.mount("/api", api::v4::routes())
		.mount("/v4", api::v4::routes())
}

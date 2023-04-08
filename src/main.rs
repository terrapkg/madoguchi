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
use opentelemetry_sdk::{trace::config, Resource};
use rocket::*;
use rocket_db_pools::Database;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[get("/")]
async fn index() -> response::Redirect {
	response::Redirect::to("https://terra.fyralabs.com/")
}

#[get("/health")]
async fn health() -> &'static str {
	"."
}

fn chks() {
	assert!(std::env::var("JWT_KEY").is_ok(), "JWT_KEY cannot be empty.");
}

async fn migrate(rocket: Rocket<Build>) -> fairing::Result {
	match db::Madoguchi::fetch(&rocket) {
		Some(db) => match sqlx::migrate!().run(&**db).await {
			Ok(_) => Ok(rocket),
			Err(e) => {
				error!("Fail to init db: {e}");
				Err(rocket)
			},
		},
		None => Err(rocket),
	}
}

#[launch]
async fn rocket() -> _ {
	dotenv::dotenv().ok();
	let tracer = opentelemetry_sdk::export::trace::stdout::new_pipeline()
		.with_pretty_print(true)
		.with_trace_config(config().with_resource(Resource::empty()))
		.install_simple();
	let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
	let file = std::fs::File::create("mg.log").unwrap_or_else(|e| panic!("Can't make log: {e:?}"));
	let flog = tracing_subscriber::fmt::layer().with_writer(std::sync::Arc::new(file));
	let sub = fmt().compact().without_time().finish();
	sub.with(telemetry).with(EnvFilter::from_default_env()).with(flog).init();
	chks();
	info!("Launching rocket 🚀");
	rocket::build()
		.attach(db::Madoguchi::init())
		.attach(rocket::fairing::AdHoc::try_on_ignite("Migrations", migrate))
		.mount("/", routes![index, health])
		.mount("/redirect", api::repology::routes())
		.mount("/ci", api::ci::routes())
		.mount("/api", api::v4::routes())
		.mount("/v4", api::v4::routes())
}

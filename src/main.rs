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
use tracing::{error, info, instrument::WithSubscriber};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

#[get("/")]
async fn index() -> response::Redirect {
	response::Redirect::to("https://terra.fyralabs.com/")
}

#[get("/health")]
async fn health() -> &'static str {
	env!("CARGO_PKG_VERSION")
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
	opentelemetry::global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
	let tracer = opentelemetry_jaeger::new_agent_pipeline()
		.with_endpoint("localhost:3200")
		.with_service_name("madoguchi")
		.with_max_packet_size(9216)
		.with_auto_split_batch(true)
		.install_batch(opentelemetry::runtime::Tokio)
		.expect("Cannot build/install tracer");
	let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
	Registry::default()
		.with(telemetry)
		.with(EnvFilter::from_default_env())
		.with(tracing_logfmt::layer())
		.init();
	chks();
	info!("Launching rocket 🚀");
	rocket::build()
		.attach(db::Madoguchi::init())
		.attach(rocket::fairing::AdHoc::try_on_ignite("Migrations", migrate))
		.attach(rocket::fairing::AdHoc::on_shutdown("OpenTelemetry", |_| {
			Box::pin(async move {
				opentelemetry::global::shutdown_tracer_provider();
			})
		}))
		.mount("/", routes![index, health])
		.mount("/redirect", api::repology::routes())
		.mount("/ci", api::ci::routes())
		.mount("/api", api::v4::routes())
		.mount("/v4", api::v4::routes())
}

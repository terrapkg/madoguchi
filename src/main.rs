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
use rocket::*;
use rocket_db_pools::Database;

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
				error!("Fail to init db: {}", e);
				Err(rocket)
			},
		},
		None => Err(rocket),
	}
}

#[launch]
async fn rocket() -> _ {
	dotenv::dotenv().ok();
	chks();
	rocket::build()
		.attach(db::Madoguchi::init())
		.attach(rocket::fairing::AdHoc::try_on_ignite("Migrations", migrate))
		.mount("/", routes![index, health])
		.mount("/redirect", api::repology::routes())
		.mount("/ci", api::ci::routes())
		.mount("/api", api::api::routes())
}

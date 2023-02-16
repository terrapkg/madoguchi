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

use db::Madoguchi;
use rocket::*;
use rocket_db_pools::{Connection, Database};

#[get("/test")]
async fn test(mut db: Connection<Madoguchi>) -> Option<String> {
	sqlx::query!("SELECT * FROM repos").fetch_one(&mut *db).await.map(|record| record.name).ok()
}

fn chks() {
	assert!(std::env::var("GITHUB_TOKEN").is_ok(), "GITHUB_TOKEN is not found but required.");
	assert!(std::env::var("API_TOKEN").is_ok(), "API_TOKEN cannot be empty.");
}

#[launch]
fn rocket() -> _ {
	dotenv::dotenv().expect("dotenv didn't work?");
	chks();
	rocket::build()
		.attach(Madoguchi::init())
		.mount("/", routes![test])
		.mount("/redirect", api::repology::routes())
		.mount("/ci", api::ci::routes())
		.mount("/api", api::api::routes())
}

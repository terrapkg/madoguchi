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
mod db;

use crate::db::Repo;
use db::Madoguchi;
use rocket::*;
use rocket_db_pools::{
	sqlx::{Acquire, Executor, Row},
	Connection, Database,
};
use sqlx::{Postgres, Transaction};

#[get("/test")]
async fn test(mut db: Connection<Madoguchi>) -> Option<String> {
	let mut t: Transaction<Postgres> = db.begin().await.ok()?;
	sqlx::query!("SELECT * FROM repos").fetch_one(&mut *t).await.map(|record| record.name).ok()
}

#[launch]
fn rocket() -> _ {
	rocket::build().attach(Madoguchi::init()).mount("/", routes![test])
}

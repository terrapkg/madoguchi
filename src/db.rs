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
use rocket_db_pools::{sqlx::PgPool, Database};
use serde::Serialize;

#[derive(Database)]
#[database("madoguchi")]
pub struct Madoguchi(PgPool);

#[derive(sqlx::FromRow, Serialize)]
pub struct Repo {
	pub name: String,
	pub link: String,
	pub gh: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Pkg {
	pub name: String,
	pub repo: String,
	pub verl: String,
	pub arch: String,
	pub dirs: String,
	pub build: Option<String>,
}

#[derive(sqlx::FromRow)]
pub struct Build {
	pub id: String,
	pub epoch: sqlx::types::chrono::NaiveDateTime,
	pub pname: String,
	pub pverl: String,
	pub parch: String,
	pub repo: String,
}

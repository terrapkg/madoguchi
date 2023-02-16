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

#[derive(Database)]
#[database("madoguchi")]
pub struct Madoguchi(PgPool);

pub struct Repo {
	pub name: String,
	pub link: String,
	pub gh: String,
}

pub struct Pkg {
	pub name: String,
	pub repo: String,
	pub verl: String,
	pub arch: String,
	pub dirs: String,
	pub build: Option<i64>,
}

pub struct Build {
	pub id: i32,
	pub epoch: sqlx::types::chrono::NaiveDateTime,
	pub pname: String,
	pub pverl: String,
	pub parch: String,
	pub repo: String,
	pub runid: String,
}

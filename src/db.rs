//! This file is part of Madoguchi.
//!
//! Madoguchi is free software: you can redistribute it and/or modify it under the terms of
//! the GNU General Public License as published by the Free Software Foundation, either
//! version 3 of the License, or (at your option) any later version.
//!
//! Madoguchi is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
//! without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
//! See the GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License along with Madoguchi.
//! If not, see <https://www.gnu.org/licenses/>.
//!
// sqlx bug
#![allow(clippy::option_if_let_else, clippy::renamed_function_params)]
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

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct Pkg {
	pub name: String,
	pub repo: String,
	pub ver: String,
	pub rel: String,
	pub arch: String,
	pub dirs: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct Build {
	pub id: String,
	pub epoch: chrono::NaiveDateTime,
	pub pname: String,
	pub pver: String,
	pub prel: String,
	pub parch: String,
	pub repo: String,
	pub succ: bool,
	pub commit: Option<String>,
}

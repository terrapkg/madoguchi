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
use crate::db::{Madoguchi as Mg, Build};
use rocket::response::Redirect;
use rocket::{get, routes, Route};
use rocket_db_pools::Connection;
use sqlx::types::chrono;

pub(crate) fn routes() -> Vec<Route> {
	routes![]
}

#[get("/<repo>/add/b/<name>?<verl>&<arch>&<dirs>&<link>")]
async fn add_bulid(mut db: Connection<Mg>, repo: String, name: String, verl: String, arch: String, dirs: String, link: String) -> Option<()> {
    // track the build and see if it passes
    let epoch = chrono::Utc::now().naive_utc();
    // let b = Build {
    //     pname: name, pverl: verl, parch: arch, link, epoch
    // };
    // insert b;
    // dunno
    None
}

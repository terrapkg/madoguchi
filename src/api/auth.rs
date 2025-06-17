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
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use jwt_simple::prelude::*;
use rocket::{
	http::Status,
	request::{self, FromRequest},
};
use std::sync::LazyLock;

pub static JWT_KEY: LazyLock<HS256Key> = LazyLock::new(|| {
	HS256Key::from_bytes(&STANDARD_NO_PAD.decode(std::env::var("JWT_KEY").unwrap()).unwrap())
});

pub struct ApiAuth {
	pub token: String,
}
#[derive(Debug)]
pub enum ApiError {
	Nil,
	NoAdminScope,
}
impl std::fmt::Display for ApiError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Nil => write!(f, "Fail to verify token"),
			Self::NoAdminScope => write!(f, "Token has no admin scope as required"),
		}
	}
}
impl std::error::Error for ApiError {}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiAuth {
	type Error = ApiError;
	async fn from_request(req: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
		for token in req.headers().get("Authorization").filter_map(|a| a.strip_prefix("Bearer ")) {
			let options = VerificationOptions::default();
			if let Ok(claims) = JWT_KEY.verify_token::<CustomClaims>(token, Some(options)) {
				return if claims.custom.scopes.contains(&"admin".to_owned()) {
					request::Outcome::Success(Self { token: token.to_owned() })
				} else {
					request::Outcome::Error((Status::Forbidden, ApiError::NoAdminScope))
				};
			}
		}
		request::Outcome::Error((Status::Forbidden, ApiError::Nil))
	}
}

#[derive(Serialize, Deserialize)]
struct CustomClaims {
	// Right now, this is the same as subatomic, where admin is only currently supported
	scopes: Vec<String>,
}

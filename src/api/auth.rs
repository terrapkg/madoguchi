use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use jwt_simple::prelude::*;
use rocket::{
	http::Status,
	request::{self, FromRequest},
};

lazy_static::lazy_static! {
	pub static ref JWT_KEY: HS256Key = HS256Key::from_bytes(&STANDARD_NO_PAD.decode(std::env::var("JWT_KEY").unwrap()).unwrap());
}

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
			ApiError::Nil => write!(f, "Fail to verify token"),
			ApiError::NoAdminScope => write!(f, "Token has no admin scope as required"),
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
				return if claims.custom.scopes.contains(&"admin".to_string()) {
					request::Outcome::Success(ApiAuth { token: token.to_string() })
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

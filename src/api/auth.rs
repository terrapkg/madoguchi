use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use jwt_simple::prelude::*;
use rocket::{
	http::Status,
	request::{self, FromRequest},
};

lazy_static::lazy_static! {
	pub static ref REQ: reqwest::Client = {
		use reqwest::header::*;
		let mut headers = HeaderMap::new();
		headers.append(ACCEPT, "application/vnd.github+json".parse().unwrap());
		headers.append(AUTHORIZATION, format!("Bearer {}", std::env::var("GITHUB_TOKEN").unwrap()).parse().unwrap());
		headers.append("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
		reqwest::Client::builder().default_headers(headers).build().unwrap()
	};
	pub static ref JWT_KEY: HS256Key = HS256Key::from_bytes(&STANDARD_NO_PAD.decode(std::env::var("JWT_KEY").unwrap()).unwrap());
}

pub struct ApiAuth {
	pub token: String,
}
#[derive(Debug)]
pub enum ApiError {
	Nil,
}
impl std::fmt::Display for ApiError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Random API Error")
	}
}
impl std::error::Error for ApiError {}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiAuth {
	type Error = ApiError;
	async fn from_request(req: &'r rocket::Request<'_>) -> request::Outcome<Self, Self::Error> {
		for auth in req.headers().get("Authorization") {
			if let Some(token) = auth.strip_prefix("Bearer ") {
				return request::Outcome::Success(ApiAuth { token: token.to_string() });
			}
		}
		request::Outcome::Failure((Status::Forbidden, ApiError::Nil))
	}
}

pub fn verify_token(id: &str, token: &str) -> bool {
	let mut options = VerificationOptions::default();
	options.required_subject = Some(id.to_owned());
	JWT_KEY.verify_token::<NoCustomClaims>(token, Some(options)).is_ok()
}

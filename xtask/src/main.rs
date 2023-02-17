use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use jwt_simple::prelude::*;
use std::env;

type DynError = Box<dyn std::error::Error>;

fn main() {
	if let Err(e) = try_main() {
		eprintln!("{}", e);
		std::process::exit(-1);
	}
}

fn try_main() -> Result<(), DynError> {
	let task = env::args().nth(1);
	match task.as_deref() {
		Some("generate-jwt-key") => generate_jwt_key()?,
		_ => print_help(),
	}
	Ok(())
}

fn print_help() {
	eprintln!(
		"Tasks:
generate-jwt-key            generates a JWT key for use in the JWT_KEY enviroment variable
"
	)
}

fn generate_jwt_key() -> Result<(), DynError> {
	let key = HS256Key::generate();
	println!("{}", STANDARD_NO_PAD.encode(key.to_bytes()));

	Ok(())
}

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
use async_compression::futures::bufread::GzipDecoder;
use rocket::futures::{io, AsyncReadExt, TryStreamExt};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct PrimaryXML {
	metadata: XMLMetadata,
}

#[derive(Deserialize)]
struct XMLMetadata {
	#[serde(default)]
	packages: Vec<XMLPackage>,
}
#[derive(Deserialize)]
struct XMLPackage {
	name: String,
	arch: String,
	version: XMLVer,
	summary: String,
	format: XMLFormat,
}
#[derive(Deserialize)]
struct XMLVer {
	// #[serde(rename = "@epoch")]
	// epoch: String,
	#[serde(rename = "@ver")]
	ver: String,
	#[serde(rename = "@rel")]
	rel: String,
}
#[derive(Deserialize)]
struct XMLFormat {
	#[serde(rename = "rpm:license")]
	license: String,
}
pub struct PkgInf {
	pub summary: String,
	pub license: String,
}
pub async fn parse_primary_xml(url: String) -> HashMap<(String, String, String, String), PkgInf> {
	let response = reqwest::get(url).await.unwrap();
	let reader = response
		.bytes_stream()
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
		.into_async_read();
	let br = rocket::futures::io::BufReader::new(reader);
	let mut decoder = GzipDecoder::new(br);
	let mut buf = "".into();
	decoder.read_to_string(&mut buf).await.unwrap();
	let xml: PrimaryXML = quick_xml::de::from_str(&buf).unwrap();
	let mut pkgs: HashMap<(String, String, String, String), PkgInf> = HashMap::new(); // (name, ver, rel, arch)
	for pkg in xml.metadata.packages {
		pkgs.insert(
			(pkg.name, pkg.version.ver, pkg.version.rel, pkg.arch),
			PkgInf { summary: pkg.summary, license: pkg.format.license },
		);
	}
	pkgs
}

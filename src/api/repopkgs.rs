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
use tracing::{debug, error, instrument};

#[derive(Deserialize)]
struct PrimaryXML {
	#[serde(rename = "package")]
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
	// #[serde(rename = "rpm:license")]
	license: String,
}
#[derive(Debug)]
pub struct PkgInf {
	pub summary: String,
	pub license: String,
}
#[instrument]
pub async fn parse_primary_xml(
	url: String,
) -> Option<HashMap<(String, String, String, String), PkgInf>> {
	let mut primaryurl = String::new();
	debug!("Grabbing repomd.xml");
	match reqwest::get(format!("{url}/repodata/repomd.xml")).await {
		Ok(resp) => {
			for line in resp.text().await.unwrap().lines() {
				if line.ends_with(r#"-primary.xml.gz"/>"#) {
					primaryurl = line
						.trim_start()
						.strip_prefix("<location href=\"")
						.unwrap()
						.strip_suffix("\"/>")
						.unwrap()
						.to_string();
					break;
				}
			}
			if primaryurl.is_empty() {
				error!("Cannot parse repomd");
				return None;
			}
			primaryurl = format!("{url}/{primaryurl}");
		},
		Err(_) => {
			error!("Cannot fetch repomd");
			return None;
		},
	};
	debug!(primaryurl, "Grabbing primary");
	let response = reqwest::get(primaryurl).await.unwrap();
	let reader = response
		.bytes_stream()
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
		.into_async_read();
	let br = io::BufReader::new(reader);
	let mut decoder = GzipDecoder::new(br);
	let mut buf = "".into();
	decoder.multiple_members(true);
	if let Err(e) = decoder.read_to_string(&mut buf).await {
		error!(?e, "while reading compressed data to string");
		return None;
	}
	debug!(buf);
	let xml: PrimaryXML = match quick_xml::de::from_str(&buf) {
		Ok(o) => o,
		Err(e) => {
			error!(?e);
			return None;
		},
	};
	let mut pkgs: HashMap<(String, String, String, String), PkgInf> = HashMap::new(); // (name, ver, rel, arch)
	for pkg in xml.packages {
		pkgs.insert(
			(pkg.name, pkg.version.ver, pkg.version.rel, pkg.arch),
			PkgInf { summary: pkg.summary, license: pkg.format.license },
		);
	}
	tracing::trace!(?pkgs);
	Some(pkgs)
}

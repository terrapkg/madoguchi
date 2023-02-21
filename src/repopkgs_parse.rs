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
use rocket::{futures::{
	channel::mpsc::{Receiver, Sender},
	io::{self},
	AsyncRead, AsyncReadExt, SinkExt, StreamExt, TryStreamExt,
}, tokio::{task::block_in_place, runtime::Handle}};
use serde::Deserialize;
use std::io::{BufReader, Write};

pub async fn get_primary_xml(url: String, mut tx: Sender<Vec<u8>>) {
	let response = reqwest::get(url).await.unwrap();
	let reader = response
		.bytes_stream()
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
		.into_async_read();
	let br = rocket::futures::io::BufReader::new(reader);
	let mut decoder = GzipDecoder::new(br);
	loop {
		let mut buf = vec![];
		decoder.read(&mut buf).await.unwrap();
		tx.send(buf).await.unwrap();
	}
}

struct XMLReader {
	rx: Receiver<Vec<u8>>,
}

impl AsyncRead for XMLReader {
	fn poll_read(
		mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, mut buf: &mut [u8],
	) -> std::task::Poll<std::io::Result<usize>> {
		if self.rx.poll_next_unpin(cx).is_ready() {
			let mut l = 0;
			let x = self.rx.poll_next_unpin(cx).map(|r| {
				let x = r.unwrap();
				buf.write_all(&x).unwrap();
				l = x.len();
			});
			if x.is_ready() {
				std::task::Poll::Ready(Ok(l))
			} else {
				std::task::Poll::Pending
			}
		} else {
			std::task::Poll::Pending
		}
	}
}

impl std::io::Read for XMLReader {
	fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
		if let Some(x) = block_in_place(|| {
			Handle::current().block_on(self.rx.next())
		}) {
			buf.write(&x)
		} else {
			Err(std::io::ErrorKind::Other.into())
		}
	}
}

#[derive(Deserialize)]
struct PrimaryXML {}

pub async fn parse_primary_xml(rx: Receiver<Vec<u8>>) {
	let xmlreader = XMLReader { rx };
	let reader = BufReader::new(xmlreader);
	// let mut xml = Reader::from_reader(reader);
	let xml: PrimaryXML = quick_xml::de::from_reader(reader).unwrap();
}

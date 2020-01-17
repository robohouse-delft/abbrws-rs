use std::io::Write;
use hyper::Client;
use hyper::body::HttpBody;
use abbrws::DigestAuthCache;

#[tokio::main]
async fn main() {
	let client = Client::new();
	let url = "http://192.168.0.5/rw/iosystem/signals/";

	let mut auth = DigestAuthCache::new("Default User".to_owned(), "robotics".to_owned());
	let make_request = || hyper::Request::get(url).body(Default::default()).unwrap();
	let response = auth.request(&client, make_request).await.unwrap();

	let body = body(response).await.unwrap();
	std::io::stdout().write_all(&body).unwrap();
}

async fn body(response: hyper::Response<hyper::Body>) -> Result<Vec<u8>, hyper::Error> {
	let mut body = response.into_body();
	let mut data = Vec::with_capacity(512);
	while let Some(chunk) = body.data().await {
		data.extend(chunk?.as_ref());
	}

	Ok(data)
}

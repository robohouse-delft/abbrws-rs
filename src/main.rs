// use std::io::Write;
// use hyper::Client;
// use hyper::body::HttpBody;

#[tokio::main]
async fn main() {
	let mut client = abbrws::Client::new(hyper::Client::new(), "192.168.0.5", "Default User", "robotics").unwrap();

	client.get_signals().await.unwrap();
	client.get_signals().await.unwrap();

// 	let mut auth = DigestAuthCache::new("Default User".to_owned(), "robotics".to_owned());
// 	let make_request = || hyper::Request::get(url).body(Default::default()).unwrap();
// 	let response = auth.request(&client, make_request).await.unwrap();

// 	let body = body(response).await.unwrap();
// 	std::io::stdout().write_all(&body).unwrap();
}

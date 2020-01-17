// use std::io::Write;
// use hyper::Client;
// use hyper::body::HttpBody;

#[tokio::main]
async fn main() {
	let mut client = abbrws::Client::new(hyper::Client::new(), "192.168.0.5", "Default User", "robotics").unwrap();

	client.get_signals().await.unwrap();
	client.get_signals().await.unwrap();
}

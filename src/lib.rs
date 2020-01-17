mod digest_auth_cache;
use digest_auth_cache::DigestAuthCache;
use hyper::body::HttpBody;

use std::convert::TryFrom;

pub struct Client<C> {
	root_url: http::Uri,
	auth_cache: DigestAuthCache,
	http_client: hyper::Client<C, hyper::Body>,
}

#[derive(Clone, Debug)]
pub struct InvalidStatusError {
	status: http::StatusCode,
}

#[derive(Debug)]
pub enum Error {
	InvalidStatus(InvalidStatusError),
	InvalidUri(http::uri::InvalidUri),
	Http(http::Error),
	Hyper(hyper::Error),
}

impl<C> Client<C>
where
	C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
	pub fn new(http_client: hyper::Client<C, hyper::Body>, host: impl AsRef<str>, user: impl Into<String>, password: impl Into<String>) -> Result<Self, http::uri::InvalidUri> {
		let root_url = hyper::Uri::try_from(format!("http://{}/", host.as_ref()))?;
		Ok(Self {
			root_url,
			http_client,
			auth_cache: DigestAuthCache::new(user.into(), password.into()),
		})
	}

	pub async fn login(&mut self) -> Result<(), Error> {
		let url : http::Uri = format!("{}/?json=1", self.root_url).parse().unwrap();
		let request = move || {
			hyper::Request::builder()
				.uri(url.clone())
				.method(hyper::Method::GET)
				.body(hyper::Body::empty())
		};

		let response = self.auth_cache.request(&mut self.http_client, request).await?;
		if response.status() != http::StatusCode::OK {
			return Err(InvalidStatusError {
				status: response.status(),
			}.into())
		}

		let body = collect_body(response).await?;
		println!("body: {}", String::from_utf8(body).unwrap());
		todo!();
	}
}


async fn collect_body(response: hyper::Response<hyper::Body>) -> Result<Vec<u8>, hyper::Error> {
	let mut body = response.into_body();
	let mut data = Vec::with_capacity(512);
	while let Some(chunk) = body.data().await {
		data.extend(chunk?.as_ref());
	}

	Ok(data)
}

impl std::fmt::Display for InvalidStatusError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "unexpected status code: {}", self.status)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::InvalidUri(e)    => e.fmt(f),
			Self::InvalidStatus(e) => e.fmt(f),
			Self::Http(e)          => e.fmt(f),
			Self::Hyper(e)         => e.fmt(f),
		}
	}
}

impl std::error::Error for InvalidStatusError {}
impl std::error::Error for Error {}

impl From<InvalidStatusError> for Error {
	fn from(other: InvalidStatusError) -> Self {
		Self::InvalidStatus(other)
	}
}

impl From<http::uri::InvalidUri> for Error {
	fn from(other: http::uri::InvalidUri) -> Self {
		Self::InvalidUri(other)
	}
}

impl From<http::Error> for Error {
	fn from(other: http::Error) -> Self {
		Self::Http(other)
	}
}

impl From<hyper::Error> for Error {
	fn from(other: hyper::Error) -> Self {
		Self::Hyper(other)
	}
}

impl From<digest_auth_cache::RequestError> for Error {
	fn from(other: digest_auth_cache::RequestError) -> Self {
		match other {
			digest_auth_cache::RequestError::Http(e)  => Self::from(e),
			digest_auth_cache::RequestError::Hyper(e) => Self::from(e),
		}
	}
}

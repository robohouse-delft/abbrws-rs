mod digest_auth_cache;
use cookie::Cookie;
use cookie::CookieJar;
use digest_auth_cache::DigestAuthCache;
use hyper::body::HttpBody;

use std::convert::TryFrom;

pub struct Client<C> {
	root_url: http::Uri,
	auth_cache: DigestAuthCache,
	http_client: hyper::Client<C, hyper::Body>,
	cookies: cookie::CookieJar,
}

#[derive(Clone, Debug)]
pub struct InvalidStatusError {
	status: http::StatusCode,
	body: serde_json::Value,
}

#[derive(Debug)]
pub enum Error {
	InvalidStatus(InvalidStatusError),
	InvalidUri(http::uri::InvalidUri),
	Http(http::Error),
	Hyper(hyper::Error),
	Json(serde_json::Error),
	InvalidHeader(hyper::header::ToStrError),
	InvalidCookie(cookie::ParseError),
}

impl<C> Client<C>
where
	C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
	pub fn new(http_client: hyper::Client<C, hyper::Body>, host: impl AsRef<str>, user: impl Into<String>, password: impl Into<String>) -> Result<Self, http::uri::InvalidUri> {
		let root_url = hyper::Uri::try_from(format!("http://{}", host.as_ref()))?;
		Ok(Self {
			root_url,
			http_client,
			auth_cache: DigestAuthCache::new(user.into(), password.into()),
			cookies: cookie::CookieJar::new(),
		})
	}

	pub async fn login(&mut self) -> Result<(), Error> {
		let url = format!("{}/?json=1", self.root_url).parse().unwrap();
		let body = self.get(url).await?;
		Ok(())
	}

	pub async fn get_signals(&mut self) -> Result<(), Error> {
		let url = format!("{}/rw/iosystem/signals?json=1", self.root_url).parse().unwrap();
		let body = self.get(url).await?;
		eprintln!("got signals");
		Ok(())
	}

	pub async fn get_signal(&mut self, signal: impl AsRef<str>) -> Result<(), Error> {
		let url = format!("{}/rw/iosystem/signal/{}/?json=1", self.root_url, signal.as_ref()).parse().unwrap();
		let body = self.get(url).await?;
		println!("body: {:#?}", body);
		todo!();
	}

	async fn get(&mut self, url: http::Uri) -> Result<serde_json::Value, Error> {
		// Copy cookies into a list of HeaderValue objects.
		let cookie_headers : Vec<_> = self.cookies.iter().map(|cookie| {
			// Unwrap should be fine, we already parsed it from a HeaderValue earlier.
			let value = format!("{}={}", cookie.name(), cookie.value());
			hyper::header::HeaderValue::from_str(&value).unwrap()
		}).collect();

		// Add cookies.
		let request = move || {
			let mut request = hyper::Request::get(url.clone()).body(hyper::Body::empty())?;
			for cookie in &cookie_headers {
				request.headers_mut().append(hyper::header::COOKIE, cookie.clone());
			}
			Ok(request)
		};

		// Perform request.
		let response = self.auth_cache.request(&mut self.http_client, request).await?;

		// Parse cookies.
		let headers = response.headers();
		for cookie in headers.get_all(hyper::header::SET_COOKIE) {
			let cookie = cookie.to_str()?.to_string();
			self.cookies.add(Cookie::parse(cookie)?);
		}

		let status = response.status();
		let body = collect_body(response).await?;
		let body = serde_json::from_slice(&body)?;

		if status != http::StatusCode::OK {
			return Err(InvalidStatusError {
				status,
				body,
			}.into())
		}

		Ok(body)
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
			Self::Json(e)          => e.fmt(f),
			Self::InvalidHeader(e) => e.fmt(f),
			Self::InvalidCookie(e) => e.fmt(f),
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

impl From<serde_json::Error> for Error {
	fn from(other: serde_json::Error) -> Self {
		Self::Json(other)
	}
}

impl From<hyper::header::ToStrError> for Error {
	fn from(other: hyper::header::ToStrError) -> Self {
		Self::InvalidHeader(other)
	}
}

impl From<cookie::ParseError> for Error {
	fn from(other: cookie::ParseError) -> Self {
		Self::InvalidCookie(other)
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

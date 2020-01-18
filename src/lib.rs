mod digest_auth_cache;
use cookie::Cookie;
use cookie::CookieJar;
use digest_auth_cache::DigestAuthCache;
use hyper::body::HttpBody;

use std::convert::TryFrom;

mod error;
pub use error::Error;
pub use error::InvalidStatusError;
pub use error::RemoteFailureError;

mod parse;
pub use parse::signal::Signal;
pub use parse::signal::SignalKind;
pub use parse::signal::SignalValue;

pub struct Client<C> {
	root_url: http::Uri,
	auth_cache: DigestAuthCache,
	http_client: hyper::Client<C, hyper::Body>,
	cookies: CookieJar,
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
			cookies: CookieJar::new(),
		})
	}

	pub async fn login(&mut self) -> Result<(), Error> {
		let url = format!("{}/?json=1", self.root_url).parse().unwrap();
		self.get(url).await?;
		Ok(())
	}

	pub async fn get_signals(&mut self) -> Result<Vec<Signal>, Error> {
		let url = format!("{}/rw/iosystem/signals?json=1", self.root_url).parse().unwrap();
		let body = self.get(url).await?;
		Ok(parse::signal::parse_list(&body)?.map_err(|e| RemoteFailureError {
			code: Some(e.code),
			message: e.message,
		})?)
	}

	pub async fn get_signal(&mut self, signal: impl AsRef<str>) -> Result<Signal, Error> {
		let url = format!("{}/rw/iosystem/signal/{}/?json=1", self.root_url, signal.as_ref()).parse().unwrap();
		let body = self.get(url).await?;
		Ok(parse::signal::parse_one(&body)?.map_err(|e| RemoteFailureError {
			code: Some(e.code),
			message: e.message,
		})?)
	}

	async fn get(&mut self, url: http::Uri) -> Result<Vec<u8>, Error> {
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

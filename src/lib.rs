mod digest_auth_cache;
use cookie::Cookie;
use cookie::CookieJar;
use digest_auth_cache::DigestAuthCache;
use hyper::body::HttpBody;
use mime::Mime;

use std::convert::TryFrom;

mod error;
pub use error::Error;
pub use error::RemoteFailureError;
pub use error::MalformedContentTypeError;
pub use error::UnexpectedContentTypeError;

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
		Ok(parse::signal::parse_list(&body)?)
	}

	pub async fn get_signal(&mut self, signal: impl AsRef<str>) -> Result<Signal, Error> {
		let url = format!("{}/rw/iosystem/signal/{}/?json=1", self.root_url, signal.as_ref()).parse().unwrap();
		let body = self.get(url).await?;
		Ok(parse::signal::parse_one(&body)?)
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

		let http_status = response.status();
		let content_type = get_content_type(&response)?;

		if http_status == http::StatusCode::OK {
			if content_type.essence_str() == "application/json" {
				Ok(collect_body(response).await?)
			} else {
				Err(UnexpectedContentTypeError { actual: content_type, expected: "application/json".into() }.into())
			}
		} else {
			match content_type.essence_str() {
				"text/plain" => {
					Err(plain_text_to_error(http_status, collect_body(response).await?).into())
				},
				"application/json" => {
					let error = parse::parse_error(&collect_body(response).await?)?;
					Err(RemoteFailureError { http_status, code: Some(error.code), message: error.message }.into())
				},
				_ => Err(UnexpectedContentTypeError { actual: content_type, expected: "application/json or text/plain".into() }.into()),
			}
		}
	}
}

fn get_content_type<B>(response: &hyper::Response<B>) -> Result<Mime, MalformedContentTypeError> {
	let content_type = response.headers().get(hyper::header::CONTENT_TYPE)
		.ok_or(MalformedContentTypeError { content_type: Vec::new() })?;

	let make_error = || MalformedContentTypeError {
		content_type: content_type.as_bytes().to_owned(),
	};

	let content_type = content_type.to_str().map_err(|_| make_error())?;
	content_type.parse().map_err(|_| make_error())
}

fn plain_text_to_error(http_status: hyper::StatusCode, body: Vec<u8>) -> RemoteFailureError {
	let message = String::from_utf8(body).ok()
		.filter(|x| x.len() <= 150)
		.unwrap_or_default();

	RemoteFailureError {
		http_status,
		code: None,
		message
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

mod digest_auth_cache;
use cookie::Cookie;
use cookie::CookieJar;
use digest_auth_cache::DigestAuthCache;
use hyper::body::HttpBody;
use std::convert::TryFrom;

pub use mime::Mime;

mod error;
pub use error::Error;
pub use error::RemoteFailureError;
pub use error::MalformedContentTypeError;
pub use error::UnexpectedContentTypeError;

mod parse;
pub use parse::file_service::DirEntry;
pub use parse::file_service::Directory;
pub use parse::file_service::File;
pub use parse::signal::Signal;
pub use parse::signal::SignalKind;
pub use parse::signal::SignalValue;

mod url_encode;
use url_encode::url_encode_query_value;

pub struct Client<C = hyper::client::HttpConnector> {
	root_url: http::Uri,
	auth_cache: DigestAuthCache,
	http_client: hyper::Client<C, hyper::Body>,
	cookies: CookieJar,
}

type Request = hyper::Request<hyper::Body>;

/// ABB RWS client
///
/// The client manages a session with an ABB RobotWare controller.
impl<C> Client<C>
where
	C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
{
	/// Create a new client.
	pub fn new(host: impl AsRef<str>, user: impl Into<String>, password: impl Into<String>) -> Result<Self, http::uri::InvalidUri>
	where
		hyper::Client<C>: Default,
	{
		Self::new_with_http_client(Default::default(), host, user, password)
	}

	/// Create a new client using an existing [`hyper::Client`] for the HTTP requests.
	pub fn new_with_http_client(http_client: hyper::Client<C, hyper::Body>, host: impl AsRef<str>, user: impl Into<String>, password: impl Into<String>) -> Result<Self, http::uri::InvalidUri> {
		let root_url = hyper::Uri::try_from(format!("http://{}", host.as_ref()))?;
		Ok(Self {
			root_url,
			http_client,
			auth_cache: DigestAuthCache::new(user.into(), password.into()),
			cookies: CookieJar::new(),
		})
	}

	/// Establish a session with the server.
	///
	/// It is not required to call this function,
	/// since a session will be established automatically when needed,
	/// However, this allows you to avoids the overhead of digest authentication later on.
	///
	/// It can also be used to test the communication with the server.
	///
	/// Note that the session may still expire if the server decides so,
	/// in which case a new session will be established automatically.
	pub async fn login(&mut self) -> Result<(), Error> {
		let url = format!("{}/?json=1", self.root_url).parse().unwrap();
		self.get(url).await?;
		Ok(())
	}

	/// Get a list of all signals on the robot, inclusing their current status.
	pub async fn get_signals(&mut self) -> Result<Vec<Signal>, Error> {
		let url = format!("{}/rw/iosystem/signals?json=1", self.root_url).parse().unwrap();
		let (content_type, body) = self.get(url).await?;
		check_content_type(content_type, mime::APPLICATION_JSON)?;
		Ok(parse::signal::parse_list(&body)?)
	}

	/// Get the details for a single signal.
	///
	/// You can use [`get_signals`] to get a list of all available signals.
	pub async fn get_signal(&mut self, signal: impl AsRef<str>) -> Result<Signal, Error> {
		let url = format!("{}/rw/iosystem/signals/{}/?json=1", self.root_url, signal.as_ref()).parse().unwrap();
		let (content_type, body) = self.get(url).await?;
		check_content_type(content_type, mime::APPLICATION_JSON)?;
		Ok(parse::signal::parse_one(&body)?)
	}

	/// Set the value of a signal.
	///
	/// You can use [`get_signals`] to get a list of all available signals.
	pub async fn set_signal(&mut self, signal: impl AsRef<str>, value: SignalValue) -> Result<(), Error> {
		let url : http::Uri = format!("{}/rw/iosystem/signals/{}/?action=set&json=1", self.root_url, signal.as_ref()).parse().unwrap();
		let data = format!("lvalue={}", value);
		self.post_form(url, data).await?;
		Ok(())
	}

	/// List the files in a directory.
	pub async fn list_files(&mut self, directory: &str) -> Result<Vec<DirEntry>, Error> {
		let url : http::Uri = format!("{}/fileservice/{}/?json=1", self.root_url, directory).parse().unwrap();
		let (content_type, body) = self.get(url).await?;
		check_content_type(content_type, mime::APPLICATION_JSON)?;
		Ok(parse::file_service::parse_directory_listing(&body)?)
	}

	/// Create a directory.
	pub async fn create_directory(&mut self, directory: &str) -> Result<(), Error> {
		let (parent, child) = rpartition(directory, '/');
		let url : http::Uri = format!("{}/fileservice/{}/?json=1", self.root_url, parent).parse().unwrap();
		let data = format!("fs-newname={}&fs-action=create", url_encode_query_value(child));
		self.post_form(url, data).await?;
		Ok(())
	}

	/// Download a file from the controller.
	pub async fn download_file(&mut self, path: &str) -> Result<(Mime, Vec<u8>), Error> {
		let url : http::Uri = format!("{}/fileservice/{}/?json=1", self.root_url, path).parse().unwrap();
		let body = self.get(url).await?;
		// TODO: Check that it wasn't a directory somehow.
		// Hopefully we can use the content type. Needs experimenting.
		Ok(body)
	}

	/// Upload a file to the controller.
	pub async fn upload_file(&mut self, path: &str, content_type: Mime, data: impl Into<Vec<u8>>) -> Result<(), Error> {
		let url : http::Uri = format!("{}/fileservice/{}/?json=1", self.root_url, path).parse().unwrap();
		self.put(url, content_type, data).await?;
		Ok(())
	}

	/// Perform a GET request.
	async fn get(&mut self, url: http::Uri) -> Result<(Mime, Vec<u8>), Error> {
		self.request(|| hyper::Request::get(url.clone()).body(hyper::Body::empty())).await
	}

	/// Perform a POST request with form data.
	async fn post_form(&mut self, url: http::Uri, data: impl Into<Vec<u8>>) -> Result<(Mime, Vec<u8>), Error> {
		let data = data.into();
		self.request(move || hyper::Request::post(url.clone())
			.header(hyper::header::CONTENT_TYPE, "application/x-www-form-urlencoded")
			.body(data.clone().into())
		).await
	}

	/// Perform a POST request with form data.
	async fn put(&mut self, url: http::Uri, content_type: Mime, data: impl Into<Vec<u8>>) -> Result<(Mime, Vec<u8>), Error> {
		let data = data.into();
		self.request(move || hyper::Request::post(url.clone())
			.header(hyper::header::CONTENT_TYPE, content_type.as_ref())
			.body(data.clone().into())
		).await
	}

	/// Perform a HTTP request.
	///
	/// This function takes care of HTTP digest authentication and cookies.
	async fn request(&mut self, mut make_request: impl FnMut() -> http::Result<Request>) -> Result<(Mime, Vec<u8>), Error> {
		// Copy cookies into a list of HeaderValue objects.
		let cookie_headers : Vec<_> = self.cookies.iter().map(|cookie| {
			// Unwrap should be fine, we already parsed it from a HeaderValue earlier.
			let value = format!("{}={}", cookie.name(), cookie.value());
			hyper::header::HeaderValue::from_str(&value).unwrap()
		}).collect();

		let make_request = move || {
			let mut request = make_request()?;
			for cookie in &cookie_headers {
				request.headers_mut().append(hyper::header::COOKIE, cookie.clone());
			}
			Ok(request)
		};

		// Perform request.
		let response = self.auth_cache.request(&mut self.http_client, make_request).await?;

		// Parse cookies.
		let headers = response.headers();
		for cookie in headers.get_all(hyper::header::SET_COOKIE) {
			let cookie = cookie.to_str()?.to_string();
			self.cookies.add(Cookie::parse(cookie)?);
		}

		let http_status = response.status();
		let content_type = get_content_type(&response)?;

		if http_status.is_success() {
			Ok((content_type, collect_body(response).await?))
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

fn check_content_type(actual: Mime, expected: Mime) -> Result<(), UnexpectedContentTypeError> {
	if actual.essence_str() == expected.essence_str() {
		Ok(())
	} else {
		Err(UnexpectedContentTypeError { actual: actual, expected: String::from(expected.essence_str()) }.into())
	}
}

/// Get a mime type from the ContentType header of an HTTP Response.
fn get_content_type<B>(response: &hyper::Response<B>) -> Result<Mime, MalformedContentTypeError> {
	let content_type = response.headers().get(hyper::header::CONTENT_TYPE)
		.ok_or(MalformedContentTypeError { content_type: Vec::new() })?;

	let make_error = || MalformedContentTypeError {
		content_type: content_type.as_bytes().to_owned(),
	};

	let content_type = content_type.to_str().map_err(|_| make_error())?;
	content_type.parse().map_err(|_| make_error())
}

/// Convert a plain text HTTP response to a RemoteFailureError.
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

/// Collect the body of an HTTP response into a vector.
async fn collect_body(response: hyper::Response<hyper::Body>) -> Result<Vec<u8>, hyper::Error> {
	let mut body = response.into_body();
	let mut data = Vec::with_capacity(512);
	while let Some(chunk) = body.data().await {
		data.extend(chunk?.as_ref());
	}

	Ok(data)
}

fn rpartition(input: &str, pat: char) -> (&str, &str) {
	match input.rfind(pat) {
		Some(n) => (&input[..n], &input[n + 1..]),
		None => (&input, &input[input.len()..]),
	}
}

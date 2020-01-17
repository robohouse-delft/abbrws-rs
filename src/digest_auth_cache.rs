use hyper::Body;
use hyper::Client;
use hyper::Request;
use hyper::Response;
use hyper::body::HttpBody;
use hyper::client::connect::Connect;
use hyper::header::HeaderValue;

/// Cache for HTTP digest authentication.
///
/// The cache can be used to perform requests,
/// while caching the digest authentication challenge from the server.
pub struct DigestAuthCache {
	username: String,
	password: String,
	challenge: Option<digest_auth::WwwAuthenticateHeader>,
}

/// An error that can be returned by a request through the cache.
#[derive(Debug)]
pub enum RequestError {
	Hyper(hyper::Error),
	Http(http::Error),
}

impl DigestAuthCache {
	/// Create an empty digest auth cache.
	pub fn new(username: String, password: String) -> Self {
		Self {
			username,
			password,
			challenge: None,
		}
	}

	/// Perform a request using the given client.
	///
	/// If a cached challenge is available,
	/// an Authorization header is added to the request containing a response to the challenge.
	///
	/// If a request fails with status 401 Unauthorized,
	/// the response is checked for a WWW-Authenticate header containing a new challenge.
	/// The new challenge is then cached, and the request is retried with the new challenge.
	pub async fn request<C, B, BuildRequest>(
		&mut self,
		client: &Client<C, B>,
		mut build_request: BuildRequest,
	) -> Result<Response<Body>, RequestError>
	where
		BuildRequest: FnMut() -> http::Result<Request<B>>,
		C: Connect + Clone + Send + Sync + 'static,
		B: HttpBody + Send + 'static,
		<B as HttpBody>::Data : Send,
		<B as HttpBody>::Error : Into<Box<(dyn std::error::Error + Send + Sync + 'static)>>,
	{
		// Try the request with possibly cached challenge / response.
		let mut request = build_request()?;
		self.add_digest_auth(&mut request);
		let response = client.request(request).await?;

		// If we get a 401 Unauthorized, try to get a new challenge from the response.
		if response.status() != hyper::StatusCode::UNAUTHORIZED {
			return Ok(response);
		}

		// The challenge should be in the WWW-Authenticate header.
		let challenge = response.headers().get(hyper::header::WWW_AUTHENTICATE).map(|x| x.to_str());
		let challenge = match challenge {
			Some(Ok(x)) => x,
			Some(Err(_)) | None => return Ok(response),
		};

		// Parse the header and update the cached challenge.
		self.challenge = match digest_auth::parse(challenge) {
			Ok(x) => Some(x),
			Err(_) => return Ok(response),
		};

		// Retry request with new Authorization header.
		let mut request = build_request()?;
		self.add_digest_auth(&mut request);
		Ok(client.request(request).await?)
	}

	/// If a cached challenge is available, add an Authorization header to the request.
	///
	/// Returns true if the header was added, false otherwise.
	fn add_digest_auth<B>(&mut self, request: &mut Request<B>) -> bool {
		let challenge = match self.challenge.as_mut() {
			Some(x) => x,
			None => return false,
		};

		let context = digest_auth::AuthContext {
			username: self.username.as_str().into(),
			password: self.password.as_str().into(),
			uri: request.uri().path().into(),
			method: convert_method(request.method()),
			body: None,
			cnonce: None,
		};

		let answer = challenge.respond(&context).map(|x| HeaderValue::from_str(&x.to_header_string()));
		let answer = match answer {
			Ok(Ok(x)) => x,
			_ => return false,
		};
		request.headers_mut().insert(hyper::header::AUTHORIZATION, answer);
		true
	}
}

fn convert_method(method: &hyper::Method) -> digest_auth::HttpMethod {
	match method {
		&hyper::Method::GET => digest_auth::HttpMethod::GET,
		&hyper::Method::POST => digest_auth::HttpMethod::POST,
		&hyper::Method::HEAD => digest_auth::HttpMethod::HEAD,
		&hyper::Method::PUT => digest_auth::HttpMethod::OTHER("PUT"),
		&hyper::Method::DELETE => digest_auth::HttpMethod::OTHER("DELETE"),
		&hyper::Method::PATCH => digest_auth::HttpMethod::OTHER("PATCH"),
		&hyper::Method::CONNECT => digest_auth::HttpMethod::OTHER("CONNECT"),
		&hyper::Method::TRACE => digest_auth::HttpMethod::OTHER("TRACE"),
		x => panic!("unsupported HTTP method: {}", x.as_ref()),
	}
}


impl From<hyper::Error> for RequestError {
	fn from(other: hyper::Error) -> Self {
		Self::Hyper(other)
	}
}

impl From<http::Error> for RequestError {
	fn from(other: http::Error) -> Self {
		Self::Http(other)
	}
}

impl std::fmt::Display for RequestError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Hyper(e) => e.fmt(f),
			Self::Http(e) => e.fmt(f),
		}
	}
}

impl std::error::Error for RequestError {}

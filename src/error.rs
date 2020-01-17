#[derive(Clone, Debug)]
pub struct InvalidStatusError {
	pub status: http::StatusCode,
	pub body: serde_json::Value,
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

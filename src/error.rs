#[derive(Clone, Debug)]
pub struct RemoteFailureError {
	pub http_status: hyper::StatusCode,
	pub code: Option<u32>,
	pub message: String,
}

#[derive(Debug)]
pub enum Error {
	RemoteFailure(RemoteFailureError),
	InvalidUri(http::uri::InvalidUri),
	Http(http::Error),
	Hyper(hyper::Error),
	Json(serde_json::Error),
	InvalidHeader(hyper::header::ToStrError),
	InvalidCookie(cookie::ParseError),
}

impl std::fmt::Display for RemoteFailureError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "remote call failed with HTTP status {}", self.http_status)?;

		if let Some(code) = self.code {
			write!(f, "and code 0x{:08X}", code)?;
		}

		if !self.message.is_empty() {
			write!(f, ": {}", self.message)?;
		}

		Ok(())
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::RemoteFailure(e) => e.fmt(f),
			Self::InvalidUri(e)    => e.fmt(f),
			Self::Http(e)          => e.fmt(f),
			Self::Hyper(e)         => e.fmt(f),
			Self::Json(e)          => e.fmt(f),
			Self::InvalidHeader(e) => e.fmt(f),
			Self::InvalidCookie(e) => e.fmt(f),
		}
	}
}

impl std::error::Error for RemoteFailureError {}
impl std::error::Error for Error {}

impl From<RemoteFailureError> for Error {
	fn from(other: RemoteFailureError) -> Self {
		Self::RemoteFailure(other)
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

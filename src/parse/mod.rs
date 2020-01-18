use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error;

pub mod signal;

#[derive(Clone, Debug, Deserialize)]
pub struct ErrorStatus {
	#[serde(deserialize_with = "deserialize_error_code")]
	pub code: u32,

	#[serde(rename = "msg")]
	pub message: String,
}

fn deserialize_error_code<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
	// The code field of error messages is wrongly encoded as signed integer.
	// This function fixes it by casting it back to an unsigned integer.
	let value = i32::deserialize(deserializer)?;
	Ok(value as u32)
}

#[derive(Clone, Debug, Deserialize)]
struct OuterMessage<T> {
	_embedded: InnerMessage<T>,
}

#[derive(Clone, Debug, Deserialize)]
struct InnerMessage<T> {
	_state: T,
	status: Option<ErrorStatus>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum OneOrNone<T> {
	One((T, )),
	None([(); 0]),
}

impl<T> OneOrNone<T> {
	fn into_option(self) -> Option<T> {
		match self {
			Self::One((x, )) => Some(x),
			Self::None(_) => None,
		}
	}
}

pub fn parse<'a, T>(data: &'a [u8]) -> Result<Result<T, ErrorStatus>, serde_json::Error>
where
	T: Deserialize<'a>,
{
	let outer : OuterMessage<Option<T>> = serde_json::from_slice(data)?;
	if let Some(error) = outer._embedded.status {
		Ok(Err(error))
	} else if let Some(inner) = outer._embedded._state {
		Ok(Ok(inner))
	} else {
		Err(serde_json::Error::missing_field("_state"))
	}
}

pub fn parse_one<'a, T>(data: &'a [u8]) -> Result<Result<T, ErrorStatus>, serde_json::Error>
where
	T: Deserialize<'a>,
{
	let outer : OuterMessage<OneOrNone<T>> = serde_json::from_slice(data)?;
	if let Some(error) = outer._embedded.status {
		Ok(Err(error))
	} else if let Some(inner) = outer._embedded._state.into_option() {
		Ok(Ok(inner))
	} else {
		Err(serde_json::Error::missing_field("_state"))
	}
}

use serde::Deserialize;
use serde::Deserializer;

pub mod file_service;
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
	_embedded: T,
}

#[derive(Clone, Debug, Deserialize)]
struct InnerMessage<T> {
	_state: T,
}

#[derive(Clone, Debug, Deserialize)]
struct InnerErrorMessage {
	status: ErrorStatus,
}

pub fn parse_error(data: &[u8]) -> Result<ErrorStatus, serde_json::Error>
{
	let outer : OuterMessage<InnerErrorMessage> = serde_json::from_slice(data)?;
	Ok(outer._embedded.status)
}

pub fn parse_vec<'a, T>(data: &'a [u8]) -> Result<Vec<T>, serde_json::Error>
where
	T: Deserialize<'a>,
{
	let outer : OuterMessage<InnerMessage<Vec<T>>> = serde_json::from_slice(data)?;
	Ok(outer._embedded._state)
}

pub fn parse_one<'a, T>(data: &'a [u8]) -> Result<T, serde_json::Error>
where
	T: Deserialize<'a>,
{
	let outer : OuterMessage<InnerMessage<(T, )>> = serde_json::from_slice(data)?;
	Ok(outer._embedded._state.0)
}

#[cfg(test)]
mod test {
	use super::*;
	use assert2::assert;

	#[test]
	fn test_parse_bad_signal() {
		assert!(let Ok(ErrorStatus { code: 0xc0048409, .. }) = parse_error(include_bytes!("../../../samples/bad_signal.json")));
	}
}

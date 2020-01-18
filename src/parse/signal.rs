use serde::Deserialize;
use super::ErrorStatus;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
pub enum SignalKind {
	#[serde(rename = "DI")]
	DigitalInput,

	#[serde(rename = "DO")]
	DigitalOutput,

	#[serde(rename = "AI")]
	AnalogInput,

	#[serde(rename = "AO")]
	AnalogOutput,

	#[serde(rename = "GI")]
	GroupInput,

	#[serde(rename = "GO")]
	GroupOutput,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SignalValue {
	Binary(bool),
	Analog(f64),
	Group(u64),
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
struct RawSingleSignal<'a> {
	#[serde(rename = "_title")]
	pub title: &'a str,

	#[serde(rename = "type")]
	pub kind: SignalKind,

	pub category: &'a str,

	pub lvalue: &'a str,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
struct RawListSignal<'a> {
	#[serde(rename = "_title")]
	pub title: &'a str,

	#[serde(rename = "type")]
	pub kind: SignalKind,

	pub category: &'a str,

	pub lvalue: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
	pub title: String,

	pub kind: SignalKind,

	pub category: String,

	pub lvalue: SignalValue,
}

impl Signal {
	fn from_single_raw(raw: RawSingleSignal) -> serde_json::Result<Self> {
		use serde::de::Unexpected;
		use serde::de::Error;

		// Parse value depending on signal type.
		let value = match raw.kind {
			SignalKind::AnalogInput | SignalKind::AnalogOutput => {
				raw.lvalue.parse::<f64>()
					.map(|x| SignalValue::Analog(x))
					.map_err(|_| Error::invalid_type(Unexpected::Str(raw.lvalue), &"floating-point value"))
			},
			SignalKind::DigitalInput | SignalKind::DigitalOutput => {
				match raw.lvalue {
					"1" => Ok(SignalValue::Binary(true)),
					"0" => Ok(SignalValue::Binary(false)),
					_   => Err(Error::invalid_type(Unexpected::Str(raw.lvalue), &"1 or 0"))
				}
			},
			SignalKind::GroupInput | SignalKind::GroupOutput => {
				raw.lvalue.parse::<u64>()
					.map(|x| SignalValue::Group(x))
					.map_err(|_| Error::invalid_type(Unexpected::Str(raw.lvalue), &"integer"))
			},
		}?;

		Ok(Signal {
			title: raw.title.into(),
			kind: raw.kind,
			category: raw.category.into(),
			lvalue: value,
		})
	}

	fn from_list_raw(raw: RawListSignal) -> serde_json::Result<Self> {
		use serde::de::Unexpected;
		use serde::de::Error;

		// Parse value depending on signal type.
		let value = match raw.kind {
			SignalKind::AnalogInput | SignalKind::AnalogOutput => Ok(SignalValue::Analog(raw.lvalue)),
			SignalKind::DigitalInput | SignalKind::DigitalOutput => {
				if raw.lvalue == 1.0 {
					Ok(SignalValue::Binary(true))
				} else if raw.lvalue == 0.0 {
					Ok(SignalValue::Binary(false))
				} else {
					Err(Error::invalid_type(Unexpected::Float(raw.lvalue), &"1 or 0"))
				}
			},
			SignalKind::GroupInput | SignalKind::GroupOutput => {
				// TODO: Is this safe? What's the maximum number of signals in a group,
				// and do they fit lossless in a f64?
				Ok(SignalValue::Group(raw.lvalue as u64))
			},
		}?;

		Ok(Signal {
			title: raw.title.into(),
			kind: raw.kind,
			category: raw.category.into(),
			lvalue: value,
		})
	}
}

pub fn parse_one(data: &[u8]) -> serde_json::Result<Result<Signal, ErrorStatus>> {
	let raw : RawSingleSignal = match super::parse_one(data) {
		Ok(Ok(x)) => x,
		Err(e) => return Err(e),
		Ok(Err(e)) => return Ok(Err(e)),
	};

	Ok(Ok(Signal::from_single_raw(raw)?))
}

pub fn parse_list(data: &[u8]) -> serde_json::Result<Result<Vec<Signal>, ErrorStatus>> {
	let raw : Vec<RawListSignal> = match super::parse(data) {
		Ok(Ok(x)) => x,
		Err(e) => return Err(e),
		Ok(Err(e)) => return Ok(Err(e)),
	};

	let mut out = Vec::with_capacity(raw.len());
	for raw in raw {
		out.push(Signal::from_list_raw(raw)?);
	}
	Ok(Ok(out))
}

#[cfg(test)]
mod test {
	use super::*;
	use assert2::assert;
	use assert2::check;

	#[test]
	fn test_parse_signals() {
		let parsed = parse_list(include_bytes!("../../samples/signals.json"));
		assert!(let Ok(Ok(_)) = &parsed);
		let parsed = parsed.unwrap().unwrap();
	}

	#[test]
	fn test_parse_bad_signal() {
		assert!(let Ok(Err(ErrorStatus { code: 0xc0048409, msg: _ })) = parse_one(include_bytes!("../../samples/bad_signal.json")));
		assert!(let Ok(Err(ErrorStatus { code: 0xc0048409, msg: _ })) = parse_list(include_bytes!("../../samples/bad_signal.json")));
	}

	#[test]
	fn test_parse_signal() {
		let parsed = match parse_one(include_bytes!("../../samples/good_signal.json")) {
			Ok(Ok(x)) => x,
			err => { assert!(let Ok(Ok(_)) = err); panic!(); },
		};

		check!(parsed.title    == "Local/PANEL/SS2");
		check!(parsed.category == "safety");
		check!(parsed.kind     == SignalKind::DigitalInput);
		check!(parsed.lvalue   == SignalValue::Binary(true));
	}
}

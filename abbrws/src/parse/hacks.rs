//! This module contains hacks around the weird data formats used by ABB RWS.

use serde::Deserializer;
use std::convert::TryFrom;

pub trait DeserializeThroughStr: Sized {
	fn deserialize_through_str<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

impl DeserializeThroughStr for usize {
	fn deserialize_through_str<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_any(VisitThroughStr::<usize>::default())
	}
}

impl DeserializeThroughStr for bool {
	fn deserialize_through_str<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_any(VisitThroughStr::<bool>::default())
	}
}

pub fn deserialize_through_str<'de, D: Deserializer<'de>, T: DeserializeThroughStr>(deserializer: D) -> Result<T, D::Error> {
	T::deserialize_through_str(deserializer)
}

/// Visitor that parses values either directly or from a string.
#[derive(Default)]
struct VisitThroughStr<T> {
	_phantom: std::marker::PhantomData<T>,
}

impl<'de> serde::de::Visitor<'de> for VisitThroughStr<usize> {
	type Value = usize;

	fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "unsigned integer")
	}

	fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<usize, E> {
		value.parse().map_err(|_| E::invalid_value(serde::de::Unexpected::Str(value), &"unsigned integer"))
	}

	serde::serde_if_integer128! {
		fn visit_u128<E: serde::de::Error>(self, value: u128) -> Result<usize, E> {
			usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
		}

		fn visit_i128<E: serde::de::Error>(self, value: i128) -> Result<usize, E> {
			usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
		}
	}

	fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<usize, E> {
		usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
	}

	fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<usize, E> {
		usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
	}

	fn visit_u32<E: serde::de::Error>(self, value: u32) -> Result<usize, E> {
		usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
	}

	fn visit_i32<E: serde::de::Error>(self, value: i32) -> Result<usize, E> {
		usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
	}

	fn visit_u16<E: serde::de::Error>(self, value: u16) -> Result<usize, E> {
		usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
	}

	fn visit_i16<E: serde::de::Error>(self, value: i16) -> Result<usize, E> {
		usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
	}

	fn visit_u8<E: serde::de::Error>(self, value: u8) -> Result<usize, E> {
		usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
	}

	fn visit_i8<E: serde::de::Error>(self, value: i8) -> Result<usize, E> {
		usize::try_from(value).map_err(|_| E::custom(format!("value out of range for usize: {}", value)))
	}
}

impl<'de> serde::de::Visitor<'de> for VisitThroughStr<bool> {
	type Value = bool;

	fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "boolean")
	}

	fn visit_borrowed_str<E: serde::de::Error>(self, value: &str) -> Result<bool, E> {
		value.parse().map_err(|_| E::invalid_value(serde::de::Unexpected::Str(value), &"unsigned integer"))
	}

	fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<bool, E> {
		Ok(value)
	}
}

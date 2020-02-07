use serde::Deserialize;

// fn deserialize_number_from_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<usize, D::Error> {
// 	use serde::de::Error;

// 	if let Ok(x) = usize::deserialize(deserializer) {
// 		return Ok(x);
// 	}

// 	let string = deserializer.deserialize_str()?;
// 	string.parse().map_err(|e| D::Error::unexpected(serde::de::Unexpected::Str(string), "unsigned integer"))
// }

use crate::parse::hacks::deserialize_through_str;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
pub struct Device {
	#[serde(rename = "_title")]
	name: String,

	#[serde(rename = "fs-device-type")]
	device_type: String,

	#[serde(rename = "fs-free-space")]
	#[serde(deserialize_with = "deserialize_through_str")]
	free_space: usize,

	#[serde(rename = "fs-total-space")]
	#[serde(deserialize_with = "deserialize_through_str")]
	total_space: usize,

	#[serde(rename = "fs-enabled")]
	#[serde(deserialize_with = "deserialize_through_str")]
	enabled: bool,

	#[serde(rename = "fs-readonly")]
	#[serde(deserialize_with = "deserialize_through_str")]
	read_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
pub struct Directory {
	#[serde(rename = "_title")]
	name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
pub struct File {
	#[serde(rename = "_title")]
	name: String,

	#[serde(rename = "fs-cdate")]
	cdate: String,

	#[serde(rename = "fs-mdate")]
	mdate: String,

	#[serde(rename = "fs-size")]
	#[serde(deserialize_with = "deserialize_through_str")]
	size: usize,

	#[serde(rename = "fs-readonly")]
	#[serde(deserialize_with = "deserialize_through_str")]
	read_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
#[serde(tag = "_type")]
pub enum DirEntry {
	#[serde(rename = "fs-dir")]
	Directory(Directory),

	#[serde(rename = "fs-file")]
	File(File),

	#[serde(rename = "fs-device")]
	Device(Device),
}

pub fn parse_directory_listing(data: &[u8]) -> Result<Vec<DirEntry>, serde_json::Error> {
	super::parse_vec::<DirEntry>(data)
}

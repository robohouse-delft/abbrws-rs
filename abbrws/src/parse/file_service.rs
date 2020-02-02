use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
pub struct Directory {
	#[serde(rename = "title")]
	name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
pub struct File {
	#[serde(rename = "title")]
	name: String,

	#[serde(rename = "fs-cdate")]
	cdate: String,

	#[serde(rename = "fs-mdate")]
	mdate: String,

	#[serde(rename = "fs-size")]
	size: usize,

	#[serde(rename = "fs-readonly")]
	read_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
#[serde(tag = "class")]
pub enum DirEntry {
	#[serde(rename = "fs-dir")]
	Directory(Directory),

	#[serde(rename = "fs-file")]
	File(File),
}

pub fn parse_directory_listing(data: &[u8]) -> Result<Vec<DirEntry>, serde_json::Error> {
	super::parse_vec::<DirEntry>(data)
}

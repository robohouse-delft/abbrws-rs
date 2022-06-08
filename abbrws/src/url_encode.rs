
pub struct UrlEncodeQueryValue<'a> {
	data: &'a [u8],
}

fn must_escape(byte: u8) -> bool {
	match byte {
		b'#' => true,
		b'%' => true,
		b'&' => true,
		b'=' => true,
		_ => byte > 127,
	}
}

impl std::fmt::Display for UrlEncodeQueryValue<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let mut remaining = self.data;
		loop {
			// Search for something that needs escaping.
			if let Some(safe_till) = remaining.iter().position(|x| must_escape(*x)) {
				write!(f, "{}", unsafe { std::str::from_utf8_unchecked(&remaining[..safe_till]) })?;
				write!(f, "%{:02X}", remaining[safe_till])?;
				remaining = &remaining[safe_till + 1..];
			// Nothing to escape, just write it all at once.
			} else {
				write!(f, "{}", unsafe { std::str::from_utf8_unchecked(remaining) })?;
				break;
			}
		}
		Ok(())
	}
}

pub fn url_encode_query_value<T: AsRef<[u8]> + ?Sized>(data: &T) -> UrlEncodeQueryValue {
	UrlEncodeQueryValue { data: data.as_ref() }
}

#[cfg(test)]
mod test {
	use super::*;
	use assert2::assert;

	#[test]
	fn test_url_encode_query_value() {
		assert!("aap" == format!("{}", url_encode_query_value("aap")));
		assert!("%23" == format!("{}", url_encode_query_value("#")));
		assert!("%25" == format!("{}", url_encode_query_value("%")));
		assert!("%26" == format!("{}", url_encode_query_value("&")));
		assert!("%3D" == format!("{}", url_encode_query_value("=")));
		assert!("%23%25%26%3D" == format!("{}", url_encode_query_value("#%&=")));
		assert!("aap" == format!("{}", url_encode_query_value("aap")));
		assert!("aap%3Dnoot" == format!("{}", url_encode_query_value("aap=noot")));
		assert!("aap%3Dnoot%26" == format!("{}", url_encode_query_value("aap=noot&")));
		assert!("%23%25%26%3Daap%23%25%26%3Dmies%23%25%26%3D" == format!("{}", url_encode_query_value("#%&=aap#%&=mies#%&=")));
	}
}

pub mod value;

use core::fmt;
use core::iter::Iterator;
use core::result;
use std::error;

pub use value::FromValue;
pub use value::KeyVal;
pub use value::Simple;
pub use value::Value;
pub use value::ValueInto;

pub type Result<T> = result::Result<T, CborError>;
pub enum ErrorKind {
	UnexpectedValue,
	InsufficientBytes,
}

impl fmt::Debug for ErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		match self {
			ErrorKind::UnexpectedValue => f.write_str("Unexpected value"),
			ErrorKind::InsufficientBytes => f.write_str("Insufficient bytes"),
		}
	}
}

impl fmt::Display for ErrorKind {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ErrorKind::UnexpectedValue => write!(fmt, "Unexpected value"),
			ErrorKind::InsufficientBytes => write!(fmt, "Insufficient bytes"),
		}
	}
}

#[derive(Debug)]
pub struct CborError {
	kind: ErrorKind,
	error: Box<dyn error::Error + Send + Sync>,
}

impl CborError {
	fn new(kind: ErrorKind, error: Box<dyn error::Error + Send + Sync>) -> CborError {
		CborError { kind, error }
	}

	fn new_err<T>(kind: ErrorKind, error: Box<dyn error::Error + Send + Sync>) -> Result<T> {
		Err(CborError::new(kind, error))
	}
}

impl fmt::Display for CborError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		self.kind.fmt(fmt)
	}
}

impl error::Error for CborError {
	fn description(&self) -> &str {
		match self.kind {
			ErrorKind::UnexpectedValue => "Unexpected value",
			ErrorKind::InsufficientBytes => "Insufficient bytes",
		}
	}
	fn cause(&self) -> Option<&dyn error::Error> {
		None
	}
}

fn read_type(b: u8) -> (u8, u8) {
	let major: u8 = b >> 5;
	let minor: u8 = b & 31;
	(major, minor)
}

fn parse_unsigned_int<'a, I: Iterator<Item = &'a u8>>(minor: u8, iter: &mut I) -> Result<u64> {
	if minor < 1 || minor > 27 {
		return CborError::new_err(ErrorKind::UnexpectedValue, "".into());
	}

	let int_size: usize = match minor {
		27 => 8,
		26 => 4,
		25 => 2,
		24 => 1,
		_ => return Ok(u64::from(minor)),
	};

	let mut value: u64 = 0;
	for _ in 0..int_size {
		let next = iter.next();
		let byte_val: u8 = match next {
			Some(x) => *x,
			None => return CborError::new_err(ErrorKind::InsufficientBytes, "".into()),
		};
		value <<= 8;
		value |= u64::from(byte_val);
	}
	Ok(value)
}

fn parse_negative_int<'a, I: Iterator<Item = &'a u8>>(minor: u8, iter: &mut I) -> Result<i64> {
	let val: u64 = parse_unsigned_int(minor, iter)?;
	Ok(-1 - (val as i64))
}

fn parse_byte_string<'a, I: Iterator<Item = &'a u8>>(minor: u8, iter: &mut I) -> Result<Vec<u8>> {
	let mut binary_val = Vec::<u8>::new();
	if minor == 31 {
		// indefinite length
		loop {
			let val: u8 = match iter.next() {
				Some(x) => *x,
				None => return CborError::new_err(ErrorKind::InsufficientBytes, "".into()),
			};
			if val == 0xFF {
				break;
			}
			let (_, minor) = read_type(val);
			let length: u64 = parse_unsigned_int(minor, iter)?;
			for _ in 0..length {
				let val: u8 = match iter.next() {
					Some(x) => *x,
					None => return CborError::new_err(ErrorKind::InsufficientBytes, "".into()),
				};
				binary_val.push(val);
			}
		}
	} else {
		// definite length
		let length: u64 = parse_unsigned_int(minor, iter)?;
		for _ in 0..length {
			let val: u8 = match iter.next() {
				Some(x) => *x,
				None => return CborError::new_err(ErrorKind::InsufficientBytes, "".into()),
			};
			binary_val.push(val);
		}
	}
	Ok(binary_val)
}

fn parse_utf8_string<'a, I: Iterator<Item = &'a u8>>(minor: u8, iter: &mut I) -> Result<String> {
	let binary_val = parse_byte_string(minor, iter)?;
	let string_val = match String::from_utf8(binary_val) {
		Ok(s) => s,
		Err(e) => panic!("error parsing string from vec: {}", e),
	};
	Ok(string_val)
}

pub fn parse_array<'a, I: Iterator<Item = &'a u8>>(minor: u8, iter: &mut I) -> Result<Vec<Value>> {
	let mut arr = Vec::<Value>::new();

	if minor == 31 {
		// indefinite length
		while let Some(item) = decode_next(iter)? {
			arr.push(item);
		}
	} else {
		// definite length
		let length: u64 = parse_unsigned_int(minor, iter)?;
		for _ in 0..length {
			let item: Value = decode_element(iter)?;
			arr.push(item);
		}
	}

	Ok(arr)
}

fn parse_map<'a, I: Iterator<Item = &'a u8>>(minor: u8, iter: &mut I) -> Result<Vec<KeyVal>> {
	let mut map = Vec::<KeyVal>::new(); //HashMap::<Value, Value>::new();

	if minor == 31 {
		// indefinite length
		while let Some(key) = decode_next(iter)? {
			let val: Value = decode_element(iter)?;
			map.push(KeyVal { key, val })
		}
	} else {
		// definite length
		let length: u64 = parse_unsigned_int(minor, iter)?;
		for _ in 0..length {
			let key: Value = decode_element(iter)?;
			let val: Value = decode_element(iter)?;
			map.push(KeyVal { key, val });
		}
	}

	Ok(map)
}

pub fn parse_float<'a, I: Iterator<Item = &'a u8>>(minor: u8, iter: &mut I) -> Result<f64> {
	if minor < 25 || minor > 27 {
		panic!(
			"parse_float_double: minor: {} outside acceptable bounds 1-27",
			minor
		);
	}

	let num_bytes: usize = match minor {
		27 => 8,
		26 => 4,
		25 => 2,
		_ => return CborError::new_err(ErrorKind::UnexpectedValue, "Invalid minor".into()),
	};

	let mut value: u64 = 0;
	for _ in 0..num_bytes {
		let byte_val: u8 = match iter.next() {
			Some(x) => *x,
			None => return CborError::new_err(ErrorKind::InsufficientBytes, "".into()),
		};
		value <<= 8;
		value |= u64::from(byte_val);
	}

	let f = f64::from_bits(value);
	Ok(f)
}

fn parse_simple<'a, I: Iterator<Item = &'a u8>>(minor: u8, iter: &mut I) -> Result<Simple> {
	let ret = match minor {
		0..=19 => Simple::Unassigned(minor),
		20 => Simple::False,
		21 => Simple::True,
		22 => Simple::Null,
		23 => Simple::Undefined,
		24 => match iter.next() {
			Some(x) => match x {
				32..=255 => Simple::Unassigned(*x),
				_ => CborError::new_err(ErrorKind::UnexpectedValue, "".into())?,
			},
			None => CborError::new_err(ErrorKind::InsufficientBytes, "".into())?,
		},
		_ => CborError::new_err(ErrorKind::InsufficientBytes, "".into())?,
	};

	Ok(ret)
}

fn parse_value<'a, I: Iterator<Item = &'a u8>>(iter: &mut I, type_byte: u8) -> Result<Value> {
	let (major, minor) = read_type(type_byte);

	let item: Value = match major {
		0 => Value::Unsigned(parse_unsigned_int(minor, iter)?),
		1 => Value::Negative(parse_negative_int(minor, iter)?),
		2 => Value::ByteString(parse_byte_string(minor, iter)?),
		3 => Value::Utf8String(parse_utf8_string(minor, iter)?),
		4 => Value::Array(parse_array(minor, iter)?),
		5 => Value::Map(parse_map(minor, iter)?),
		6 => {
			// ToDo: let tag = parse_unsigned_int(minor, iter);
			let type_byte: u8 = match iter.next() {
				Some(x) => *x,
				None => return CborError::new_err(ErrorKind::InsufficientBytes, "".into()),
			};
			return parse_value(iter, type_byte);
		}
		7 => {
			if minor <= 24 {
				Value::Simple(parse_simple(minor, iter)?)
			} else {
				Value::Float(parse_float(minor, iter)?)
			}
		}
		_ => {
			return CborError::new_err(
				ErrorKind::UnexpectedValue,
				"Internal error: Invalid major".into(),
			);
		}
	};

	Ok(item)
}

fn decode_next<'a, I: Iterator<Item = &'a u8>>(iter: &mut I) -> Result<Option<Value>> {
	let type_byte: u8 = match iter.next() {
		Some(x) => *x,
		None => return Err(CborError::new(ErrorKind::InsufficientBytes, "".into())),
	};

	if type_byte == 0xFF {
		return Ok(None);
	}

	match parse_value(iter, type_byte) {
		Ok(x) => Ok(Some(x)),
		Err(e) => Err(e),
	}
}

fn decode_element<'a, I: Iterator<Item = &'a u8>>(iter: &mut I) -> Result<Value> {
	let type_byte: u8 = match iter.next() {
		Some(x) => *x,
		None => return Err(CborError::new(ErrorKind::InsufficientBytes, "sfsdf".into())),
	};

	parse_value(iter, type_byte)
}

pub fn decode_iter<'a, I: Iterator<Item = &'a u8>>(iter: &mut I) -> Result<Value> {
	let type_byte: u8 = match iter.next() {
		Some(x) => *x,
		None => return Err(CborError::new(ErrorKind::InsufficientBytes, "".into())),
	};

	parse_value(iter, type_byte)
}

pub fn decode<'a, I: IntoIterator<Item = &'a u8>>(stream: I) -> Result<Value> {
	let mut iter = stream.into_iter();
	decode_iter(&mut iter)
}

pub fn decode_slice(bytes: &[u8]) -> Result<Value> {
	decode_iter(&mut bytes.iter())
}

/// Decode a given IntoIterator into a given object.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use std::collections::HashMap;
/// let bytes = &[0b1010_0010, 0b0011_1000, 0b0001_1000, 0b0110_0011, 0x61, 0x62, 0x63,
///               0b0000_0111, 0b0110_0011, 0x44, 0x45, 0x46];
/// let map: HashMap<i8, String> = cborg::decode_to(bytes).unwrap().unwrap();
/// assert_eq!("abc", map[&-25]);
/// assert_eq!("DEF", map[&7]);
/// ```
/// ```
/// let bytes = &[0b1000_0011, 11, 22, 0b0001_1000, 33];
/// let array: Vec<u32> = cborg::decode_to(bytes).unwrap().unwrap();
/// assert_eq!(11, array[0]);
/// assert_eq!(22, array[1]);
/// assert_eq!(33, array[2]);
/// ```
pub fn decode_to<'a, T, I>(stream: I) -> Result<Option<T>>
where
	T: FromValue,
	I: IntoIterator<Item = &'a u8>,
{
	let mut iter = stream.into_iter();
	let v: Value = decode_iter(&mut iter)?;
	Ok(T::from_value(v))
}

#[cfg(test)]
mod tests {
	use crate::KeyVal;
	use crate::Value;
	use crate::ValueInto;
	use core::fmt::Write;
	use std::collections::BTreeMap;
	use std::collections::HashMap;

	const LONG_STRING: &str = "This line is greater than 256 characters to test if lengths are encoded correctly after the major. This line is greater than 256 characters to test if lengths are encoded correctly after the major. This line is greater than 256 characters to test if lengths are encoded correctly after the major.";

	// TEST_DATA_DEFINITE:
	// {
	// 	555: {
	// 		"float": 2.5,
	// 		"bytestring": [0x01, 0x02, 0x03, 0x04, 0x05],
	// 		"utf8string": "你好，世界 - hello, world",
	// 		"long string": "This line is greater than 256 characters to test if lengths are encoded correctly after the major. This line is greater than 256 characters to test if lengths are encoded correctly after the major. This line is greater than 256 characters to test if lengths are encoded correctly after the major.",
	// 		"unsigned": 8,
	// 		"negative": -4
	// 	},
	// 	777: [
	// 		11,
	// 		-22,
	// 		33.3,
	// 		"fourty-four"
	// 	]
	// }
	const TEST_DATA_DEFINITE: [u8; 438] = [
		0xA2, 0x19, 0x02, 0x2B, 0xA6, 0x65, 0x66, 0x6C, 0x6F, 0x61, 0x74, 0xFB, 0x40, 0x04, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x6A, 0x62, 0x79, 0x74, 0x65, 0x73, 0x74, 0x72, 0x69, 0x6E,
		0x67, 0x45, 0x01, 0x02, 0x03, 0x04, 0x05, 0x6A, 0x75, 0x74, 0x66, 0x38, 0x73, 0x74, 0x72,
		0x69, 0x6E, 0x67, 0x78, 0x1E, 0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD, 0xEF, 0xBC, 0x8C, 0xE4,
		0xB8, 0x96, 0xE7, 0x95, 0x8C, 0x20, 0x2D, 0x20, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20,
		0x77, 0x6F, 0x72, 0x6C, 0x64, 0x6B, 0x6C, 0x6F, 0x6E, 0x67, 0x20, 0x73, 0x74, 0x72, 0x69,
		0x6E, 0x67, 0x79, 0x01, 0x28, 0x54, 0x68, 0x69, 0x73, 0x20, 0x6C, 0x69, 0x6E, 0x65, 0x20,
		0x69, 0x73, 0x20, 0x67, 0x72, 0x65, 0x61, 0x74, 0x65, 0x72, 0x20, 0x74, 0x68, 0x61, 0x6E,
		0x20, 0x32, 0x35, 0x36, 0x20, 0x63, 0x68, 0x61, 0x72, 0x61, 0x63, 0x74, 0x65, 0x72, 0x73,
		0x20, 0x74, 0x6F, 0x20, 0x74, 0x65, 0x73, 0x74, 0x20, 0x69, 0x66, 0x20, 0x6C, 0x65, 0x6E,
		0x67, 0x74, 0x68, 0x73, 0x20, 0x61, 0x72, 0x65, 0x20, 0x65, 0x6E, 0x63, 0x6F, 0x64, 0x65,
		0x64, 0x20, 0x63, 0x6F, 0x72, 0x72, 0x65, 0x63, 0x74, 0x6C, 0x79, 0x20, 0x61, 0x66, 0x74,
		0x65, 0x72, 0x20, 0x74, 0x68, 0x65, 0x20, 0x6D, 0x61, 0x6A, 0x6F, 0x72, 0x2E, 0x20, 0x54,
		0x68, 0x69, 0x73, 0x20, 0x6C, 0x69, 0x6E, 0x65, 0x20, 0x69, 0x73, 0x20, 0x67, 0x72, 0x65,
		0x61, 0x74, 0x65, 0x72, 0x20, 0x74, 0x68, 0x61, 0x6E, 0x20, 0x32, 0x35, 0x36, 0x20, 0x63,
		0x68, 0x61, 0x72, 0x61, 0x63, 0x74, 0x65, 0x72, 0x73, 0x20, 0x74, 0x6F, 0x20, 0x74, 0x65,
		0x73, 0x74, 0x20, 0x69, 0x66, 0x20, 0x6C, 0x65, 0x6E, 0x67, 0x74, 0x68, 0x73, 0x20, 0x61,
		0x72, 0x65, 0x20, 0x65, 0x6E, 0x63, 0x6F, 0x64, 0x65, 0x64, 0x20, 0x63, 0x6F, 0x72, 0x72,
		0x65, 0x63, 0x74, 0x6C, 0x79, 0x20, 0x61, 0x66, 0x74, 0x65, 0x72, 0x20, 0x74, 0x68, 0x65,
		0x20, 0x6D, 0x61, 0x6A, 0x6F, 0x72, 0x2E, 0x20, 0x54, 0x68, 0x69, 0x73, 0x20, 0x6C, 0x69,
		0x6E, 0x65, 0x20, 0x69, 0x73, 0x20, 0x67, 0x72, 0x65, 0x61, 0x74, 0x65, 0x72, 0x20, 0x74,
		0x68, 0x61, 0x6E, 0x20, 0x32, 0x35, 0x36, 0x20, 0x63, 0x68, 0x61, 0x72, 0x61, 0x63, 0x74,
		0x65, 0x72, 0x73, 0x20, 0x74, 0x6F, 0x20, 0x74, 0x65, 0x73, 0x74, 0x20, 0x69, 0x66, 0x20,
		0x6C, 0x65, 0x6E, 0x67, 0x74, 0x68, 0x73, 0x20, 0x61, 0x72, 0x65, 0x20, 0x65, 0x6E, 0x63,
		0x6F, 0x64, 0x65, 0x64, 0x20, 0x63, 0x6F, 0x72, 0x72, 0x65, 0x63, 0x74, 0x6C, 0x79, 0x20,
		0x61, 0x66, 0x74, 0x65, 0x72, 0x20, 0x74, 0x68, 0x65, 0x20, 0x6D, 0x61, 0x6A, 0x6F, 0x72,
		0x2E, 0x68, 0x75, 0x6E, 0x73, 0x69, 0x67, 0x6E, 0x65, 0x64, 0x08, 0x68, 0x6E, 0x65, 0x67,
		0x61, 0x74, 0x69, 0x76, 0x65, 0x23, 0x19, 0x03, 0x09, 0x84, 0x0B, 0x35, 0xFB, 0x40, 0x40,
		0xA6, 0x66, 0x66, 0x66, 0x66, 0x66, 0x6B, 0x66, 0x6F, 0x75, 0x72, 0x74, 0x79, 0x2D, 0x66,
		0x6F, 0x75, 0x72,
	];

	// TEST_DATA_INDEFINITE:
	// {
	// 	555: {
	// 		"float": 2.5,
	// 		"bytestring": h'0102030405',
	// 		"utf8string": "你好，世界" + " - " + "hello, world",
	// 		"unsigned": 8,
	// 		"negative": -4
	// 	},
	// 	777: [
	// 		11,
	// 		-22,
	// 		33.3,
	// 		"fourty-four"
	// 	]
	// }
	const TEST_DATA_INDEFINITE: [u8; 133] = [
		0xBF, 0x19, 0x02, 0x2B, 0xBF, 0x65, 0x66, 0x6C, 0x6F, 0x61, 0x74, 0xFB, 0x40, 0x04, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x6A, 0x62, 0x79, 0x74, 0x65, 0x73, 0x74, 0x72, 0x69, 0x6E,
		0x67, 0x45, 0x01, 0x02, 0x03, 0x04, 0x05, 0x6A, 0x75, 0x74, 0x66, 0x38, 0x73, 0x74, 0x72,
		0x69, 0x6E, 0x67, 0x7F, 0x6F, 0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD, 0xEF, 0xBC, 0x8C, 0xE4,
		0xB8, 0x96, 0xE7, 0x95, 0x8C, 0x63, 0x20, 0x2D, 0x20, 0x6C, 0x68, 0x65, 0x6C, 0x6C, 0x6F,
		0x2C, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0xFF, 0x68, 0x75, 0x6E, 0x73, 0x69, 0x67, 0x6E,
		0x65, 0x64, 0x08, 0x68, 0x6E, 0x65, 0x67, 0x61, 0x74, 0x69, 0x76, 0x65, 0x23, 0xFF, 0x19,
		0x03, 0x09, 0x9F, 0x0B, 0x35, 0xFB, 0x40, 0x40, 0xA6, 0x66, 0x66, 0x66, 0x66, 0x66, 0x6B,
		0x66, 0x6F, 0x75, 0x72, 0x74, 0x79, 0x2D, 0x66, 0x6F, 0x75, 0x72, 0xFF, 0xFF,
	];

	#[test]
	fn decode_test() {
		let arr: [&[u8]; 2] = [&TEST_DATA_DEFINITE, &TEST_DATA_INDEFINITE];
		for test_data in &arr {
			let v: Vec<u8> = test_data.to_vec();
			let data = crate::decode(&v).unwrap();

			let map: HashMap<Value, Value> = match data.get_hash_map() {
				Some(x) => x,
				None => panic!("get_map returned None"),
			}; //.expect("get_map returned None");
			let item: &Value = &map[&Value::Unsigned(555)];
			let map555 = item.get_hash_map();
			let map_inner: HashMap<Value, Value> = match map555 {
				Some(x) => x,
				None => panic!("get_map returned None"),
			};

			let float: f64 = map_inner[&Value::Utf8String("float".to_string())]
				.get_float()
				.expect("get_float returned None");
			let bytestring: Vec<u8> = map_inner[&Value::Utf8String("bytestring".to_string())]
				.get_bytes()
				.expect("get_bytes returned None");
			let utf8string: String = map_inner[&Value::Utf8String("utf8string".to_string())]
				.get_string()
				.expect("get_string returned None");
			let unsigned: u64 = map_inner[&Value::Utf8String("unsigned".to_string())]
				.get_uint()
				.expect("get_uint returned None");
			let negative: i64 = map_inner[&Value::Utf8String("negative".to_string())]
				.get_neg()
				.expect("get_int returned None");
			assert!(2.49 < float && float < 2.51);
			assert_eq!(bytestring, vec! {1,2,3,4,5});
			assert_eq!(utf8string, "你好，世界 - hello, world");
			if test_data.len() > 200 {
				// Special case - only test definite for now
				let longstring: String = map_inner[&Value::Utf8String("long string".to_string())]
					.get_string()
					.expect("get_string returned None");
				assert_eq!(longstring, LONG_STRING);
			}
			assert_eq!(unsigned, 8);
			assert_eq!(negative, -4);

			let item: &Value = &map[&Value::Unsigned(777)];
			let arr777 = item.get_array().expect("get_array returned None");
			assert_eq!(4, arr777.len());
			assert_eq!(Value::Unsigned(11), arr777[0]);
			assert_eq!(Value::Negative(-22), arr777[1]);
			assert_eq!(Value::Float(33.3), arr777[2]);
			assert_eq!(Value::Utf8String(String::from("fourty-four")), arr777[3]);
		}
	}

	#[test]
	fn encode_test() {
		let data = Value::Map(vec![
			KeyVal {
				key: Value::Unsigned(555),
				val: Value::Map(vec![
					KeyVal {
						key: Value::Utf8String(String::from("float")),
						val: Value::Float(2.5),
					},
					KeyVal {
						key: Value::Utf8String(String::from("bytestring")),
						val: Value::ByteString(vec![1, 2, 3, 4, 5]),
					},
					KeyVal {
						key: Value::Utf8String(String::from("utf8string")),
						val: Value::Utf8String(String::from("你好，世界 - hello, world")),
					},
					KeyVal {
						key: Value::Utf8String(String::from("long string")),
						val: Value::Utf8String(String::from("This line is greater than 256 characters to test if lengths are encoded correctly after the major. This line is greater than 256 characters to test if lengths are encoded correctly after the major. This line is greater than 256 characters to test if lengths are encoded correctly after the major.")),
					},
					KeyVal {
						key: Value::Utf8String(String::from("unsigned")),
						val: Value::Unsigned(8),
					},
					KeyVal {
						key: Value::Utf8String(String::from("negative")),
						val: Value::Negative(-4),
					},
				]),
			},
			KeyVal {
				key: Value::Unsigned(777),
				val: Value::Array(vec![
					Value::Unsigned(11),
					Value::Negative(-22),
					Value::Float(33.3),
					Value::Utf8String(String::from("fourty-four")),
				]),
			},
		]);

		let bytes: Vec<u8> = data.encode();
		for i in 0..125 {
			if bytes[i] != TEST_DATA_DEFINITE[i] {
				println!(
					"mismatch at pos {}: {} : {}",
					i, TEST_DATA_DEFINITE[i], bytes[i]
				);
			}
		}

		assert_eq!(TEST_DATA_DEFINITE.to_vec(), bytes);
	}

	#[test]
	fn display_test() {
		let data = crate::decode_slice(&TEST_DATA_INDEFINITE).unwrap();
		let mut out = String::new();
		write!(out, "{}", &data).expect("Could not fmt CBOR");
		assert_eq!(
			out,
			r#"{
   555: {
      "float": 2.5,
      "bytestring": [1, 2, 3, 4, 5],
      "utf8string": "你好，世界 - hello, world",
      "unsigned": 8,
      "negative": -4,
   },
   777: [
      11,
      -22,
      33.3,
      "fourty-four",
   ],
}"#
		);
	}

	#[test]
	#[allow(clippy::float_cmp)]
	fn type_test() {
		let v = crate::decode(TEST_DATA_DEFINITE.iter()).unwrap();
		let utf8_key = "utf8string";
		let utf8_val = "你好，世界 - hello, world";
		let longstring = "long string";

		let dict: HashMap<u64, HashMap<String, String>> = ValueInto::to_type(&v).unwrap();
		assert_eq!(1, dict.len());
		assert!(dict.contains_key(&555));
		let map2 = dict.get(&555).unwrap();
		assert_eq!(2, map2.len());
		let val = map2.get(utf8_key).unwrap();
		assert_eq!(utf8_val, val);

		let dict: BTreeMap<i64, HashMap<String, String>> = ValueInto::to_type(&v).unwrap();
		assert_eq!(1, dict.len());
		assert!(dict.contains_key(&555));
		let map2 = dict.get(&555).unwrap();
		assert_eq!(2, map2.len());
		let val = map2.get(utf8_key).unwrap();
		assert_eq!(utf8_val, val);

		let dict: BTreeMap<i64, Vec<i64>> = ValueInto::to_type(&v).unwrap();
		let arr = dict.get(&777).unwrap();
		assert_eq!(2, arr.len());
		assert_eq!(11, arr[0]);
		assert_eq!(-22, arr[1]);

		let dict: BTreeMap<i64, Vec<f64>> = ValueInto::to_type(&v).unwrap();
		let arr = dict.get(&777).unwrap();
		assert_eq!(3, arr.len());
		assert_eq!(11.0, arr[0]);
		assert_eq!(-22.0, arr[1]);
		assert_eq!(33.3, arr[2]);

		let dict: BTreeMap<i64, Vec<Value>> = ValueInto::to_type(&v).unwrap();
		let arr = dict.get(&777).unwrap();
		assert_eq!(4, arr.len());
		assert_eq!(11, arr[0].get_uint().unwrap());
		assert_eq!(-22, arr[1].get_neg().unwrap());
		assert_eq!(33.3, arr[2].get_float().unwrap());
		assert_eq!("fourty-four", arr[3].get_string().unwrap());

		let dict: Vec<(u64, BTreeMap<String, String>)> = ValueInto::into_type(v).unwrap();
		assert_eq!(1, dict.len());
		assert_eq!(2, dict[0].1.len());
		assert_eq!(utf8_val, dict[0].1[utf8_key]);
		assert!(dict[0].1[longstring].len() > 256);
	}

	#[test]
	#[allow(clippy::float_cmp)]
	fn decode_to_test() {
		let utf8_key = "utf8string";
		let utf8_val = "你好，世界 - hello, world";
		let longstring = "long string";

		let dict: HashMap<u64, HashMap<String, String>> = crate::decode_to(TEST_DATA_DEFINITE.iter())
			.unwrap()
			.unwrap();
		assert_eq!(1, dict.len());
		assert!(dict.contains_key(&555));
		let map2 = dict.get(&555).unwrap();
		assert_eq!(2, map2.len());
		let val = map2.get(utf8_key).unwrap();
		assert_eq!(utf8_val, val);
		assert!(map2.get(longstring).unwrap().len() > 256);

		let dict: BTreeMap<i64, Vec<i64>> = crate::decode_to(TEST_DATA_DEFINITE.iter())
			.unwrap()
			.unwrap();
		let arr = dict.get(&777).unwrap();
		assert_eq!(2, arr.len());
		assert_eq!(11, arr[0]);
		assert_eq!(-22, arr[1]);
	}
}

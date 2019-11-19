pub mod value;

use core::fmt;
use core::iter::Iterator;
use core::result;
use std::error;

pub use value::FromValue;
pub use value::KeyVal;
pub use value::Simple;
pub use value::ToValue;
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
	fn new(kind: ErrorKind, error: Box<dyn error::Error + Send + Sync>) -> CborError { CborError { kind, error } }

	fn new_err<T>(kind: ErrorKind, error: Box<dyn error::Error + Send + Sync>) -> Result<T> {
		Err(CborError::new(kind, error))
	}
}

impl fmt::Display for CborError {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result { self.kind.fmt(fmt) }
}

impl error::Error for CborError {
	fn description(&self) -> &str {
		match self.kind {
			ErrorKind::UnexpectedValue => "Unexpected value",
			ErrorKind::InsufficientBytes => "Insufficient bytes",
		}
	}
	fn cause(&self) -> Option<&dyn error::Error> { None }
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
		panic!("parse_float_double: minor: {} outside acceptable bounds 1-27", minor);
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
			return CborError::new_err(ErrorKind::UnexpectedValue, "Internal error: Invalid major".into());
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

pub fn decode_slice(bytes: &[u8]) -> Result<Value> { decode_iter(&mut bytes.iter()) }

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
	I: IntoIterator<Item = &'a u8>, {
	let mut iter = stream.into_iter();
	let v: Value = decode_iter(&mut iter)?;
	Ok(T::from_value(v))
}

/// Encode a given object into CBOR.
///
/// # Examples
///
/// Basic usage:
///
///```
/// use std::collections::HashMap;
/// let map: HashMap<u32, &str> = [
///    (33, "thirty-three"),
///    (44, "fourty-four"),
///    (55, "fifty-five")
/// ].iter().cloned().collect();
/// let cbor_bytes: Vec<u8> = cborg::encode(map);
/// ```
pub fn encode<V>(v: V) -> Vec<u8>
where
	Value: From<V>, {
	Value::from(v).encode()
}

/// Like `encode` but takes a reference.
pub fn encode_ref<V>(v: &V) -> Vec<u8>
where
	V: ToValue, {
	v.to_value().encode()
}

/// Like `encode` but takes a dynamic trait object.
pub fn encode_dyn(v: &dyn ToValue) -> Vec<u8> { v.to_value().encode() }

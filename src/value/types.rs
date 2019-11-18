use core::fmt;
use std::collections::HashMap;
use std::io;

#[derive(Clone, PartialEq, Hash)]
pub enum Simple {
	False,
	True,
	Null,
	Undefined,
	Unassigned(u8),
}

impl Simple {
	pub fn encode(&self) -> Vec<u8> {
		let major = 7 << 5;
		let mut v = Vec::<u8>::new();
		match self {
			Simple::False => v.push(major | 20),
			Simple::True => v.push(major | 21),
			Simple::Null => v.push(major | 22),
			Simple::Undefined => v.push(major | 23),
			Simple::Unassigned(x) => {
				if *x < 20 {
					v.push(major | *x);
				} else if *x > 31 {
					v.push(major | 24);
				}
			}
		}
		v
	}
}
impl std::fmt::Display for Simple {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let ss: String;
		let s: &str = match self {
			Simple::False => "false",
			Simple::True => "true",
			Simple::Null => "null",
			Simple::Undefined => "undefined",
			Simple::Unassigned(x) => {
				ss = x.to_string();
				&ss
			}
		};
		f.write_str(s)
	}
}

#[derive(Clone)] // Clone needed for get_array() to return a clone of vec
pub enum Value {
	Unsigned(u64),
	Negative(i64),
	ByteString(Vec<u8>),
	Utf8String(String),
	Array(Vec<Value>),
	Map(Vec<KeyVal>), // Vec used in place of map to preserve ordering of original data
	Float(f64),
	Simple(Simple),
}

#[derive(Clone, PartialEq)]
pub struct KeyVal {
	pub key: Value,
	pub val: Value,
}

impl Eq for Value {}
impl PartialEq for Value {
	fn eq(&self, rhs: &Self) -> bool {
		use Value::*;
		if self.major() != rhs.major() {
			return false;
		} // Unnecesary?
		match (self, rhs) {
			(Unsigned(a), Unsigned(b)) => a == b,
			(Negative(a), Negative(b)) => a == b,
			(ByteString(a), ByteString(b)) => a == b,
			(Utf8String(a), Utf8String(b)) => a == b,
			(Array(a), Array(b)) => a == b,
			(Map(a), Map(b)) => a == b,
			(Float(a), Float(b)) => a == b,
			(Simple(a), Simple(b)) => a == b,
			(Float(_), Simple(_)) => false,
			(Simple(_), Float(_)) => false,
			(_, _) => false,
		}
	}
}

impl std::hash::Hash for Value {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Value::Unsigned(x) => x.hash(state),
			Value::Negative(x) => x.hash(state),
			Value::ByteString(x) => x.hash(state),
			Value::Utf8String(x) => x.hash(state),
			Value::Array(x) => x.hash(state),
			Value::Map(x) => {
				for kv in x {
					kv.key.hash(state);
					kv.val.hash(state);
				}
			}
			Value::Float(x) => {
				let y: u64 = unsafe { std::mem::transmute::<f64, u64>(*x) };
				y.hash(state);
			}
			Value::Simple(x) => x.hash(state),
		}
	}
}

impl std::fmt::Debug for Value {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut output = Vec::<u8>::new();
		match print_cbor(&self, &mut output) {
			Ok(x) => x,
			Err(_) => return Err(fmt::Error),
		};
		let s = match std::str::from_utf8(&output) {
			Ok(s) => s,
			Err(_) => return Err(std::fmt::Error),
		};
		f.write_str(s)
	}
}

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut output = Vec::<u8>::new();
		match print_cbor(&self, &mut output) {
			Ok(x) => x,
			Err(_) => return Err(fmt::Error),
		};
		let s = match std::str::from_utf8(&output) {
			Ok(s) => s,
			Err(_) => return Err(std::fmt::Error),
		};
		f.write_str(s)
	}
}

impl Value {
	pub fn major(&self) -> u8 {
		match self {
			Self::Unsigned(_) => 0,
			Self::Negative(_) => 1,
			Self::ByteString(_) => 2,
			Self::Utf8String(_) => 3,
			Self::Array(_) => 4,
			Self::Map(_) => 5,
			// Self::Tag(_) => 6,
			Self::Float(_) => 7,
			Self::Simple(_) => 7,
		}
	}

	pub fn get_uint(&self) -> Option<u64> {
		match self {
			Value::Unsigned(x) => Some(*x),
			_ => None,
		}
	}

	pub fn get_neg(&self) -> Option<i64> {
		match self {
			Value::Negative(x) => Some(*x),
			_ => None,
		}
	}

	pub fn get_float(&self) -> Option<f64> {
		match self {
			Value::Float(x) => Some(*x),
			_ => None,
		}
	}

	pub fn get_bytes(&self) -> Option<Vec<u8>> {
		match self {
			Value::ByteString(x) => Some(x.clone()),
			_ => None,
		}
	}

	pub fn get_string(&self) -> Option<String> {
		match self {
			Value::Utf8String(x) => Some(x.clone()),
			_ => None,
		}
	}

	pub fn get_array(&self) -> Option<Vec<Value>> {
		match self {
			Value::Array(x) => Some(x.clone()),
			_ => None,
		}
	}

	pub fn get_map(&self) -> Option<Vec<KeyVal>> {
		match self {
			Value::Map(x) => Some(x.clone()),
			_ => None,
		}
	}

	pub fn get_hash_map(&self) -> Option<HashMap<Value, Value>> {
		let v: &Vec<KeyVal> = match self {
			Value::Map(x) => x,
			_ => {
				return None;
			}
		};
		let mut map = HashMap::<Value, Value>::new();

		for kv in v {
			let kv = kv.clone();
			map.insert(kv.key, kv.val);
		}

		Some(map)
	}

	fn encode_compact_uint(bytes: &mut Vec<u8>, x: u64, major: u8) {
		let mut b: u8 = major << 5;
		let byte_len;
		if x <= 23 {
			b |= x as u8;
			byte_len = 0;
		} else if x < 0xFF {
			b |= 24;
			byte_len = 1;
		} else if x < 0xFFFF {
			b |= 25;
			byte_len = 2;
		} else if x < 0xFFFF_FFFF {
			b |= 26;
			byte_len = 4;
		} else {
			b |= 27;
			byte_len = 8;
		}
		bytes.push(b);
		for i in 0..byte_len {
			bytes.push((x >> (8 * ((byte_len - 1) - i))) as u8);
		}
	}

	fn push_major_and_len(bytes: &mut Vec<u8>, len: usize, item_code: u8) {
		let length_code: u8;
		match len {
			0..=23 => {
				length_code = len as u8;
				let b: u8 = (item_code << 5) | length_code;
				bytes.push(b);
			}
			24..=0xFF => {
				length_code = 24;
				let b: u8 = (item_code << 5) | length_code;
				bytes.push(b);
				bytes.push(len as u8);
			}
			0x100..=0xFFFF => {
				length_code = 25;
				let b: u8 = (item_code << 5) | length_code;
				bytes.push(b);
				bytes.push((len >> 8) as u8);
				bytes.push(len as u8);
			}
			0x1_0000..=0xFFFF_FFFF => {
				length_code = 26;
				let b: u8 = (item_code << 5) | length_code;
				bytes.push(b);
				bytes.push((len >> 16) as u8);
				bytes.push((len >> 8) as u8);
				bytes.push(len as u8);
			}
			_ => {
				length_code = 27;
				let b: u8 = (item_code << 5) | length_code;
				bytes.push(b);
				bytes.push((len >> 24) as u8);
				bytes.push((len >> 16) as u8);
				bytes.push((len >> 8) as u8);
				bytes.push(len as u8);
			}
		};
	}

	fn add_bytes(bytes: &mut Vec<u8>, x: &[u8], item_code: u8) {
		Value::push_major_and_len(bytes, x.len(), item_code);
		for item in x {
			bytes.push(*item);
		}
	}

	pub fn encode_compact(&self) -> Vec<u8> {
		let mut bytes = Vec::<u8>::new();
		match self {
			Value::Unsigned(x) => Value::encode_compact_uint(&mut bytes, *x, 0),
			Value::Negative(x) => {
				let x: u64 = (-1 - x) as u64;
				Value::encode_compact_uint(&mut bytes, x, 1);
			}

			Value::ByteString(ref x) => {
				Value::add_bytes(&mut bytes, x.as_slice(), 2);
			}
			Value::Utf8String(ref x) => {
				Value::add_bytes(&mut bytes, x.as_bytes(), 3);
			}
			Value::Array(ref x) => {
				Value::push_major_and_len(&mut bytes, x.len(), 4);
				for item in x {
					bytes.append(&mut item.encode_compact());
				}
			}
			Value::Map(ref x) => {
				Value::push_major_and_len(&mut bytes, x.len(), 5);
				for kv in x {
					bytes.append(&mut kv.key.encode_compact());
					bytes.append(&mut kv.val.encode_compact());
				}
			}
			Value::Float(x) => {
				let b: u8 = 7 << 5 | 27;
				bytes.push(b);
				let x: u64 = x.to_bits(); // unsafe { *(x as *const f64 as *const u64) };
				for i in 0..8 {
					bytes.push((x >> (8 * (7 - i))) as u8);
				}
			}
			Value::Simple(x) => {
				for b in x.encode() {
					bytes.push(b);
				}
			}
		}
		bytes
	}

	pub fn encode(&self) -> Vec<u8> { self.encode_compact() }

	// Possible future extension
	// pub fn encode_preserving_types(&self) -> Vec<u8> {
	// 	let TODO: u8;
	// 	return Vec::<u8>::new();
	// }
}

pub fn print_cbor<W: io::Write>(val: &Value, w: &mut W) -> io::Result<()> {
	print_cbor_padded(val, 0, w)?;
	Ok(())
}

fn print_cbor_padded<W: io::Write>(val: &Value, indent: usize, w: &mut W) -> io::Result<()> {
	match val {
		Value::Unsigned(x) => write!(w, "{}", x),
		Value::Negative(x) => write!(w, "{}", x),
		Value::ByteString(ref x) => {
			if x.is_empty() {
				w.write_all(b"[]")?;
			} else if x.len() == 1 {
				write!(w, "[ {} ]", x[0])?;
			} else {
				w.write_all(b"[")?;
				write!(w, "{}", x[0])?;
				for y in x.iter().skip(1) {
					write!(w, ", {}", y)?;
				}
				w.write_all(b"]")?;
			}
			Ok(())
		}
		Value::Utf8String(ref x) => write!(w, r#""{}""#, x),
		Value::Array(ref x) => {
			w.write_all(b"[\n")?;
			for y in x {
				for _ in 0..=indent {
					w.write_all(b"   ")?;
				}
				print_cbor_padded(&y, indent, w)?;
				w.write_all(b",\n")?;
			}
			for _ in 0..indent {
				w.write_all(b"   ")?;
			}
			w.write_all(b"]")?;
			Ok(())
		}
		Value::Map(ref x) => {
			w.write_all(b"{\n")?;
			for kv in x {
				for _ in 0..=indent {
					w.write_all(b"   ")?;
				}
				print_cbor_padded(&kv.key, indent + 1, w)?;
				w.write_all(b": ")?;
				print_cbor_padded(&kv.val, indent + 1, w)?;
				w.write_all(b",\n")?;
			}
			for _ in 0..indent {
				w.write_all(b"   ")?;
			}
			w.write_all(b"}")?;
			Ok(())
		}
		Value::Float(x) => write!(w, "{}", x),
		Value::Simple(x) => write!(w, "{}", x),
	}
}

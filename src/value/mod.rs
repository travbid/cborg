pub mod types;

use core::convert::TryFrom;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub use types::KeyVal;
pub use types::Simple;
pub use types::Value;

impl TryFrom<Value> for u8 {
	type Error = ();
	fn try_from(value: Value) -> Result<u8, ()> {
		match value {
			Value::Unsigned(x) => match u8::try_from(x) {
				Ok(x) => Ok(x),
				Err(_) => Err(()),
			},
			Value::Negative(x) => match u8::try_from(x) {
				Ok(x) => Ok(x),
				Err(_) => Err(()),
			},
			_ => Err(()),
		}
	}
}

impl TryFrom<&Value> for u8 {
	type Error = ();
	fn try_from(value: &Value) -> Result<u8, ()> {
		match value {
			Value::Unsigned(x) => match u8::try_from(*x) {
				Ok(x) => Ok(x),
				Err(_) => Err(()),
			},
			Value::Negative(x) => match u8::try_from(*x) {
				Ok(x) => Ok(x),
				Err(_) => Err(()),
			},
			_ => Err(()),
		}
	}
}

pub trait FromValue {
	fn from_value(v: Value) -> Option<Self>
	where
		Self: Sized;

	fn from_ref(v: &Value) -> Option<Self>
	where
		Self: Sized;
}

impl FromValue for Value {
	fn from_value(v: Value) -> Option<Self> { Some(v) }
	fn from_ref(v: &Value) -> Option<Self> { Some(v.clone()) }
}
impl FromValue for u64 {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(x),
			Value::Negative(x) => match u64::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(*x),
			Value::Negative(x) => match u64::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
}

impl FromValue for u32 {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match u32::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match u32::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match u32::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match u32::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
}

impl FromValue for usize {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match usize::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match usize::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match usize::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match usize::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
}

impl FromValue for i64 {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match i64::try_from(x) {
				Ok(x) => Some(x as i64),
				Err(_) => None,
			},
			Value::Negative(x) => Some(x),
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match i64::try_from(*x) {
				Ok(x) => Some(x as i64),
				Err(_) => None,
			},
			Value::Negative(x) => Some(*x),
			_ => None,
		}
	}
}

impl FromValue for i32 {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match i32::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match i32::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match i32::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match i32::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
}

impl FromValue for i8 {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match i8::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match i8::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match i8::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match i8::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
}

impl FromValue for isize {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match isize::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match isize::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match isize::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			Value::Negative(x) => match isize::try_from(*x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
}
impl<K, V, S> FromValue for HashMap<K, V, S>
where
	K: FromValue + Eq + std::hash::Hash,
	V: FromValue,
	S: std::hash::BuildHasher + Default,
{
	fn from_value(v: Value) -> Option<Self> {
		let cmap: Vec<KeyVal> = match v {
			Value::Map(x) => x,
			_ => return None,
		};

		let mut m = HashMap::<K, V, S>::with_hasher(S::default());

		for kv in cmap {
			if let Some(k) = K::from_value(kv.key) {
				if let Some(v) = V::from_value(kv.val) {
					m.insert(k, v);
				}
			}
		}

		Some(m)
	}

	fn from_ref(v: &Value) -> Option<Self> {
		let cmap: &Vec<KeyVal> = match v {
			Value::Map(x) => x,
			_ => return None,
		};

		let mut m = HashMap::<K, V, S>::with_hasher(S::default());

		for kv in cmap {
			if let Some(k) = K::from_ref(&kv.key) {
				if let Some(v) = V::from_ref(&kv.val) {
					m.insert(k, v);
				}
			}
		}

		Some(m)
	}
}

impl<K, V> FromValue for BTreeMap<K, V>
where
	K: FromValue + std::cmp::Ord,
	V: FromValue,
{
	fn from_value(v: Value) -> Option<Self> {
		let cmap: Vec<KeyVal> = match v {
			Value::Map(x) => x,
			_ => return None,
		};

		let mut m = BTreeMap::<K, V>::new();

		for kv in cmap {
			if let Some(k) = K::from_value(kv.key) {
				if let Some(v) = V::from_value(kv.val) {
					m.insert(k, v);
				}
			}
		}

		Some(m)
	}
	fn from_ref(v: &Value) -> Option<Self> {
		let cmap: &Vec<KeyVal> = match v {
			Value::Map(x) => x,
			_ => return None,
		};

		let mut m = BTreeMap::<K, V>::new();

		for kv in cmap {
			if let Some(k) = K::from_ref(&kv.key) {
				if let Some(v) = V::from_ref(&kv.val) {
					m.insert(k, v);
				}
			}
		}

		Some(m)
	}
}

// Needs specialization feature in Stable
// impl FromValue for u8 {
// 	fn from_value(v: Value) -> Option<Self> {
// 		match v {
// 			Value::Unsigned(x) => match u8::try_from(x) {
// 				Ok(x) => Some(x),
// 				Err(_) => None,
// 			},
// 			Value::Negative(x) => match u8::try_from(x) {
// 				Ok(x) => Some(x),
// 				Err(_) => None,
// 			},
// 			_ => None
// 		}
// 	}
// }

impl<T> FromValue for Vec<T>
where
	T: FromValue,
{
	fn from_value(v: Value) -> Option<Self> {
		let value_arr: Vec<Value> = match v {
			Value::Array(x) => x,
			Value::Map(m) => {
				let mut arr = Vec::<T>::new();
				for kv in m {
					if let Some(x) = T::from_value(Value::Map(vec![kv.clone()])) {
						arr.push(x);
					}
				}
				return Some(arr);
			}
			_ => return None,
		};

		let mut arr = Vec::<T>::new();

		for item in value_arr {
			if let Some(x) = T::from_value(item) {
				arr.push(x);
			}
		}

		Some(arr)
	}

	fn from_ref(v: &Value) -> Option<Self> {
		let value_arr: &Vec<Value> = match v {
			Value::Array(x) => x,
			Value::Map(m) => {
				let mut arr = Vec::<T>::new();
				for kv in m {
					if let Some(x) = T::from_value(Value::Map(vec![kv.clone()])) {
						arr.push(x);
					}
				}
				return Some(arr);
			}
			_ => return None,
		};

		let mut arr = Vec::<T>::new();

		for item in value_arr {
			if let Some(x) = T::from_ref(item) {
				arr.push(x);
			}
		}

		Some(arr)
	}
}
impl FromValue for Vec<u8> {
	fn from_value(v: Value) -> Option<Self> {
		let value_arr: Vec<Value> = match v {
			Value::ByteString(bs) => return Some(bs),
			Value::Array(x) => x,
			_ => return None,
		};

		let mut arr = Vec::<u8>::new();

		for item in value_arr {
			if let Ok(x) = u8::try_from(item) {
				arr.push(x);
			}
		}

		Some(arr)
	}

	fn from_ref(v: &Value) -> Option<Self> {
		let value_arr: &Vec<Value> = match v {
			Value::ByteString(bs) => return Some(bs.clone()),
			Value::Array(x) => x,
			_ => return None,
		};

		let mut arr = Vec::<u8>::new();

		for item in value_arr {
			if let Ok(x) = u8::try_from(item) {
				arr.push(x);
			}
		}

		Some(arr)
	}
}

impl<K, V> FromValue for (K, V)
where
	K: FromValue,
	V: FromValue,
{
	fn from_value(v: Value) -> Option<Self> {
		let pair: Vec<KeyVal> = match v {
			Value::Map(m) => m,
			_ => return None,
		};

		if pair.len() != 1 {
			return None;
		}

		for kv in pair {
			if let Some(k) = K::from_value(kv.key) {
				if let Some(v) = V::from_value(kv.val) {
					return Some((k, v));
				}
			}
		}

		None
	}

	fn from_ref(v: &Value) -> Option<Self> {
		let pair: &Vec<KeyVal> = match v {
			Value::Map(m) => m,
			_ => return None,
		};

		if pair.len() != 1 {
			return None;
		}

		for kv in pair {
			if let Some(k) = K::from_ref(&kv.key) {
				if let Some(v) = V::from_ref(&kv.val) {
					return Some((k, v));
				}
			}
		}

		None
	}
}
impl FromValue for String {
	fn from_value(v: Value) -> Option<Self> { v.get_string() }
	fn from_ref(v: &Value) -> Option<Self> { v.get_string() }
}
impl FromValue for f64 {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(x as f64),
			Value::Negative(x) => Some(x as f64),
			Value::Float(x) => Some(x),
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(*x as f64),
			Value::Negative(x) => Some(*x as f64),
			Value::Float(x) => Some(*x),
			_ => None,
		}
	}
}

impl FromValue for f32 {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(x as f32),
			Value::Negative(x) => Some(x as f32),
			Value::Float(x) => Some((x) as f32),
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(*x as f32),
			Value::Negative(x) => Some(*x as f32),
			Value::Float(x) => Some(*x as f32),
			_ => None,
		}
	}
}
impl FromValue for bool {
	fn from_value(v: Value) -> Option<Self> {
		match v {
			Value::Simple(x) => match x {
				Simple::True => Some(true),
				Simple::False => Some(false),
				_ => None,
			},
			_ => None,
		}
	}
	fn from_ref(v: &Value) -> Option<Self> {
		match v {
			Value::Simple(x) => match x {
				Simple::True => Some(true),
				Simple::False => Some(false),
				_ => None,
			},
			_ => None,
		}
	}
}
// -----------------------------------------------------------------------------
pub trait ValueInto<T> {
	fn into_type(self) -> Option<T>
	where
		Self: Sized;

	fn to_type(&self) -> Option<T>
	where
		Self: Sized;
}

impl<U> ValueInto<U> for Value
where
	U: FromValue,
{
	fn into_type(self) -> Option<U> { U::from_value(self) }
	fn to_type(&self) -> Option<U> { U::from_value(self.clone()) }
}
// -----------------------------------------------------------------------------
pub trait ToValue {
	fn to_value(&self) -> Value;
}
impl ToValue for Value {
	fn to_value(&self) -> Value { self.clone() }
}
// impl ToValue for u8 {
// 	fn to_value(&self) -> Value {
// 		Value::Unsigned(u64::from(*self))
// 	}
// }
impl ToValue for u32 {
	fn to_value(&self) -> Value { Value::Unsigned(u64::from(*self)) }
}
impl ToValue for u64 {
	fn to_value(&self) -> Value { Value::Unsigned(*self) }
}
impl ToValue for i8 {
	fn to_value(&self) -> Value {
		if *self < 0 {
			Value::Negative(i64::from(*self))
		} else {
			Value::Unsigned(*self as u64)
		}
	}
}
impl ToValue for i32 {
	fn to_value(&self) -> Value {
		if *self < 0 {
			Value::Negative(i64::from(*self))
		} else {
			Value::Unsigned(*self as u64)
		}
	}
}
impl ToValue for i64 {
	fn to_value(&self) -> Value {
		if *self < 0 {
			Value::Negative(*self)
		} else {
			Value::Unsigned(*self as u64)
		}
	}
}
impl ToValue for Vec<u8> {
	fn to_value(&self) -> Value { Value::ByteString(self.clone()) }
}
impl ToValue for String {
	fn to_value(&self) -> Value { Value::Utf8String(self.clone()) }
}
impl ToValue for str {
	fn to_value(&self) -> Value { Value::Utf8String(String::from(self)) }
}
impl ToValue for &str {
	fn to_value(&self) -> Value { Value::Utf8String(String::from(*self)) }
}
impl<T> ToValue for Vec<T>
where
	T: ToValue,
{
	fn to_value(&self) -> Value {
		let mut arr = Vec::<Value>::with_capacity(self.len());
		for e in self {
			arr.push(e.to_value());
		}
		Value::Array(arr)
	}
}
impl<K, V, S> ToValue for HashMap<K, V, S>
where
	K: ToValue,
	V: ToValue,
{
	fn to_value(&self) -> Value {
		let mut v = Vec::<KeyVal>::new();
		for entry in self {
			let kv = KeyVal {
				key: entry.0.to_value(),
				val: entry.1.to_value(),
			};
			v.push(kv);
		}
		Value::Map(v)
	}
}

impl<K, V> ToValue for BTreeMap<K, V>
where
	K: ToValue,
	V: ToValue,
{
	fn to_value(&self) -> Value {
		let mut v = Vec::<KeyVal>::new();
		for entry in self {
			let kv = KeyVal {
				key: entry.0.to_value(),
				val: entry.1.to_value(),
			};
			v.push(kv);
		}
		Value::Map(v)
	}
}
impl ToValue for f32 {
	fn to_value(&self) -> Value { Value::Float(*self as f64) }
}
impl ToValue for f64 {
	fn to_value(&self) -> Value { Value::Float(*self) }
}
impl ToValue for bool {
	fn to_value(&self) -> Value {
		if *self {
			Value::Simple(Simple::True)
		} else {
			Value::Simple(Simple::False)
		}
	}
}
// -----------------------------------------------------------------------------
// impl From<u8> for Value {
// 	fn from(i: u8) -> Value {
// 		Value::Unsigned(u64::from(i))
// 	}
// }
impl From<u32> for Value {
	fn from(i: u32) -> Value { Value::Unsigned(u64::from(i)) }
}
impl From<u64> for Value {
	fn from(i: u64) -> Value { Value::Unsigned(i) }
}
impl From<i8> for Value {
	fn from(i: i8) -> Value {
		if i < 0 {
			Value::Negative(i64::from(i))
		} else {
			Value::Unsigned(i as u64)
		}
	}
}
impl From<i32> for Value {
	fn from(i: i32) -> Value {
		if i < 0 {
			Value::Negative(i64::from(i))
		} else {
			Value::Unsigned(i as u64)
		}
	}
}
impl From<i64> for Value {
	fn from(i: i64) -> Value {
		if i < 0 {
			Value::Negative(i)
		} else {
			Value::Unsigned(i as u64)
		}
	}
}
impl From<Vec<u8>> for Value {
	fn from(v: Vec<u8>) -> Self { Value::ByteString(v) }
}
impl From<String> for Value {
	fn from(s: String) -> Self { Value::Utf8String(s) }
}
impl From<&str> for Value {
	fn from(s: &str) -> Self { Value::Utf8String(String::from(s)) }
}
impl<T> From<Vec<T>> for Value
where
	Value: From<T>,
{
	fn from(v: Vec<T>) -> Self {
		let mut arr = Vec::<Value>::with_capacity(v.len());
		for e in v {
			arr.push(Value::from(e));
		}
		Value::Array(arr)
	}
}

impl<K, V, S> From<HashMap<K, V, S>> for Value
where
	Value: From<K>,
	Value: From<V>,
{
	fn from(map: HashMap<K, V, S>) -> Self {
		let mut v = Vec::<KeyVal>::new();
		for entry in map {
			let kv = KeyVal {
				key: Value::from(entry.0),
				val: Value::from(entry.1),
			};
			v.push(kv);
		}
		Value::Map(v)
	}
}

impl<K, V> From<BTreeMap<K, V>> for Value
where
	Value: From<K>,
	Value: From<V>,
{
	fn from(map: BTreeMap<K, V>) -> Self {
		let mut v = Vec::<KeyVal>::new();
		for entry in map {
			let kv = KeyVal {
				key: Value::from(entry.0),
				val: Value::from(entry.1),
			};
			v.push(kv);
		}
		Value::Map(v)
	}
}
impl From<f32> for Value {
	fn from(i: f32) -> Value { Value::Float(i as f64) }
}
impl From<f64> for Value {
	fn from(i: f64) -> Value { Value::Float(i) }
}
impl From<bool> for Value {
	fn from(i: bool) -> Value {
		if i {
			Value::Simple(Simple::True)
		} else {
			Value::Simple(Simple::False)
		}
	}
}

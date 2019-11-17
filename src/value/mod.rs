pub mod types;

use core::convert::TryFrom;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub use types::KeyVal;
pub use types::Simple;
pub use types::Value;

pub trait ValueInto<T> {
	fn into_type(self) -> Option<T>
	where
		Self: Sized;

	fn to_type(&self) -> Option<T>
	where
		Self: Sized;
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
	fn from_value(v: Value) -> Option<Self> {
		Some(v)
	}
	fn from_ref(v: &Value) -> Option<Self> {
		Some(v.clone())
	}
}

impl<U> ValueInto<U> for Value
where
	U: FromValue,
{
	fn into_type(self) -> Option<U> {
		U::from_value(self)
	}
	fn to_type(&self) -> Option<U> {
		U::from_value(self.clone())
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
		let tup: Vec<KeyVal> = match v {
			Value::Map(m) => m,
			_ => return None,
		};

		if tup.len() != 1 {
			return None;
		}

		for kv in tup {
			if let Some(k) = K::from_value(kv.key) {
				if let Some(v) = V::from_value(kv.val) {
					return Some((k, v));
				}
			}
		}

		None
	}

	fn from_ref(v: &Value) -> Option<Self> {
		let tup: &Vec<KeyVal> = match v {
			Value::Map(m) => m,
			_ => return None,
		};

		if tup.len() != 1 {
			return None;
		}

		for kv in tup {
			if let Some(k) = K::from_ref(&kv.key) {
				if let Some(v) = V::from_ref(&kv.val) {
					return Some((k, v));
				}
			}
		}

		None
	}
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

impl FromValue for String {
	fn from_value(v: Value) -> Option<Self> {
		v.get_string()
	}
	fn from_ref(v: &Value) -> Option<Self> {
		v.get_string()
	}
}

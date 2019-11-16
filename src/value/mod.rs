pub mod types;

use core::convert::TryFrom;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub use types::KeyVal;
pub use types::Simple;
pub use types::Value;

pub trait ValueInto {
	fn into_type(v: Value) -> Option<Self>
	where
		Self: Sized;

	fn to_type(v: &Value) -> Option<Self>
	where
		Self: Sized;
}

impl ValueInto for Value {
	fn into_type(v: Value) -> Option<Self> {
		Some(v)
	}
	fn to_type(v: &Value) -> Option<Self> {
		Some(v.clone())
	}
}

impl<K, V, S> ValueInto for HashMap<K, V, S>
where
	K: ValueInto + Eq + std::hash::Hash,
	V: ValueInto + Eq,
	S: std::hash::BuildHasher + Default,
{
	fn into_type(v: Value) -> Option<Self> {
		let cmap: Vec<KeyVal> = match v {
			Value::Map(x) => x,
			_ => return None,
		};

		let mut m = HashMap::<K, V, S>::with_hasher(S::default());

		for kv in cmap {
			if let Some(k) = K::into_type(kv.key) {
				if let Some(v) = V::into_type(kv.val) {
					m.insert(k, v);
				}
			}
		}

		Some(m)
	}
	fn to_type(v: &Value) -> Option<Self> {
		let cmap: &Vec<KeyVal> = match v {
			Value::Map(x) => x,
			_ => return None,
		};

		let mut m = HashMap::<K, V, S>::with_hasher(S::default());

		for kv in cmap {
			if let Some(k) = K::to_type(&kv.key) {
				if let Some(v) = V::to_type(&kv.val) {
					m.insert(k, v);
				}
			}
		}

		Some(m)
	}
}

impl<K, V> ValueInto for BTreeMap<K, V>
where
	K: ValueInto + std::cmp::Ord,
	V: ValueInto,
{
	fn into_type(v: Value) -> Option<Self> {
		let cmap: Vec<KeyVal> = match v {
			Value::Map(x) => x,
			_ => return None,
		};

		let mut m = BTreeMap::<K, V>::new();

		for kv in cmap {
			if let Some(k) = K::into_type(kv.key) {
				if let Some(v) = V::into_type(kv.val) {
					m.insert(k, v);
				}
			}
		}

		Some(m)
	}
	fn to_type(v: &Value) -> Option<Self> {
		let cmap: &Vec<KeyVal> = match v {
			Value::Map(x) => x,
			_ => return None,
		};

		let mut m = BTreeMap::<K, V>::new();

		for kv in cmap {
			if let Some(k) = K::to_type(&kv.key) {
				if let Some(v) = V::to_type(&kv.val) {
					m.insert(k, v);
				}
			}
		}

		Some(m)
	}
}

// Needs specialization feature in Stable
// impl ValueInto for u8 {
// 	fn into_val(v: Value) -> Option<Self> {
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

impl<T> ValueInto for Vec<T>
where
	T: ValueInto,
{
	fn into_type(v: Value) -> Option<Self> {
		let value_arr: Vec<Value> = match v {
			Value::Array(x) => x,
			Value::Map(m) => {
				let mut arr = Vec::<T>::new();
				for kv in m {
					if let Some(x) = T::into_type(Value::Map(vec![kv.clone()])) {
						arr.push(x);
					}
				}
				return Some(arr);
			}
			_ => return None,
		};

		let mut arr = Vec::<T>::new();

		for item in value_arr {
			if let Some(x) = T::into_type(item) {
				arr.push(x);
			}
		}

		Some(arr)
	}

	fn to_type(v: &Value) -> Option<Self> {
		let value_arr: &Vec<Value> = match v {
			Value::Array(x) => x,
			Value::Map(m) => {
				let mut arr = Vec::<T>::new();
				for kv in m {
					if let Some(x) = T::into_type(Value::Map(vec![kv.clone()])) {
						arr.push(x);
					}
				}
				return Some(arr);
			}
			_ => return None,
		};

		let mut arr = Vec::<T>::new();

		for item in value_arr {
			if let Some(x) = T::to_type(item) {
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

impl ValueInto for Vec<u8> {
	fn into_type(v: Value) -> Option<Self> {
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

	fn to_type(v: &Value) -> Option<Self> {
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

impl<K, V> ValueInto for (K, V)
where
	K: ValueInto,
	V: ValueInto,
{
	fn into_type(v: Value) -> Option<Self> {
		let tup: Vec<KeyVal> = match v {
			Value::Map(m) => m,
			_ => return None,
		};

		if tup.len() != 1 {
			return None;
		}

		for kv in tup {
			if let Some(k) = K::into_type(kv.key) {
				if let Some(v) = V::into_type(kv.val) {
					return Some((k, v));
				}
			}
		}

		None
	}

	fn to_type(v: &Value) -> Option<Self> {
		let tup: &Vec<KeyVal> = match v {
			Value::Map(m) => m,
			_ => return None,
		};

		if tup.len() != 1 {
			return None;
		}

		for kv in tup {
			if let Some(k) = K::to_type(&kv.key) {
				if let Some(v) = V::to_type(&kv.val) {
					return Some((k, v));
				}
			}
		}

		None
	}
}

impl ValueInto for u64 {
	fn into_type(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(x),
			Value::Negative(x) => match u64::try_from(x) {
				Ok(x) => Some(x),
				Err(_) => None,
			},
			_ => None,
		}
	}
	fn to_type(v: &Value) -> Option<Self> {
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

impl ValueInto for u32 {
	fn into_type(v: Value) -> Option<Self> {
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
	fn to_type(v: &Value) -> Option<Self> {
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

impl ValueInto for usize {
	fn into_type(v: Value) -> Option<Self> {
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
	fn to_type(v: &Value) -> Option<Self> {
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

impl ValueInto for i64 {
	fn into_type(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => match i64::try_from(x) {
				Ok(x) => Some(x as i64),
				Err(_) => None,
			},
			Value::Negative(x) => Some(x),
			_ => None,
		}
	}
	fn to_type(v: &Value) -> Option<Self> {
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

impl ValueInto for i32 {
	fn into_type(v: Value) -> Option<Self> {
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
	fn to_type(v: &Value) -> Option<Self> {
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

impl ValueInto for isize {
	fn into_type(v: Value) -> Option<Self> {
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
	fn to_type(v: &Value) -> Option<Self> {
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

impl ValueInto for f64 {
	fn into_type(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(x as f64),
			Value::Negative(x) => Some(x as f64),
			Value::Float(x) => Some(x),
			_ => None,
		}
	}
	fn to_type(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(*x as f64),
			Value::Negative(x) => Some(*x as f64),
			Value::Float(x) => Some(*x),
			_ => None,
		}
	}
}

impl ValueInto for f32 {
	fn into_type(v: Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(x as f32),
			Value::Negative(x) => Some(x as f32),
			Value::Float(x) => Some((x) as f32),
			_ => None,
		}
	}
	fn to_type(v: &Value) -> Option<Self> {
		match v {
			Value::Unsigned(x) => Some(*x as f32),
			Value::Negative(x) => Some(*x as f32),
			Value::Float(x) => Some(*x as f32),
			_ => None,
		}
	}
}

impl ValueInto for String {
	fn into_type(v: Value) -> Option<Self> {
		v.get_string()
	}
	fn to_type(v: &Value) -> Option<Self> {
		v.get_string()
	}
}

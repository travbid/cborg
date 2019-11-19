use core::fmt::Write;
use std::collections::BTreeMap;
use std::collections::HashMap;

use cborg::KeyVal;
use cborg::ToValue;
use cborg::Value;
use cborg::ValueInto;

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
	0xA2, 0x19, 0x02, 0x2B, 0xA6, 0x65, 0x66, 0x6C, 0x6F, 0x61, 0x74, 0xFB, 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x6A, 0x62, 0x79, 0x74, 0x65, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x45, 0x01, 0x02, 0x03, 0x04, 0x05, 0x6A,
	0x75, 0x74, 0x66, 0x38, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x78, 0x1E, 0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD, 0xEF,
	0xBC, 0x8C, 0xE4, 0xB8, 0x96, 0xE7, 0x95, 0x8C, 0x20, 0x2D, 0x20, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x77,
	0x6F, 0x72, 0x6C, 0x64, 0x6B, 0x6C, 0x6F, 0x6E, 0x67, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x79, 0x01, 0x28,
	0x54, 0x68, 0x69, 0x73, 0x20, 0x6C, 0x69, 0x6E, 0x65, 0x20, 0x69, 0x73, 0x20, 0x67, 0x72, 0x65, 0x61, 0x74, 0x65,
	0x72, 0x20, 0x74, 0x68, 0x61, 0x6E, 0x20, 0x32, 0x35, 0x36, 0x20, 0x63, 0x68, 0x61, 0x72, 0x61, 0x63, 0x74, 0x65,
	0x72, 0x73, 0x20, 0x74, 0x6F, 0x20, 0x74, 0x65, 0x73, 0x74, 0x20, 0x69, 0x66, 0x20, 0x6C, 0x65, 0x6E, 0x67, 0x74,
	0x68, 0x73, 0x20, 0x61, 0x72, 0x65, 0x20, 0x65, 0x6E, 0x63, 0x6F, 0x64, 0x65, 0x64, 0x20, 0x63, 0x6F, 0x72, 0x72,
	0x65, 0x63, 0x74, 0x6C, 0x79, 0x20, 0x61, 0x66, 0x74, 0x65, 0x72, 0x20, 0x74, 0x68, 0x65, 0x20, 0x6D, 0x61, 0x6A,
	0x6F, 0x72, 0x2E, 0x20, 0x54, 0x68, 0x69, 0x73, 0x20, 0x6C, 0x69, 0x6E, 0x65, 0x20, 0x69, 0x73, 0x20, 0x67, 0x72,
	0x65, 0x61, 0x74, 0x65, 0x72, 0x20, 0x74, 0x68, 0x61, 0x6E, 0x20, 0x32, 0x35, 0x36, 0x20, 0x63, 0x68, 0x61, 0x72,
	0x61, 0x63, 0x74, 0x65, 0x72, 0x73, 0x20, 0x74, 0x6F, 0x20, 0x74, 0x65, 0x73, 0x74, 0x20, 0x69, 0x66, 0x20, 0x6C,
	0x65, 0x6E, 0x67, 0x74, 0x68, 0x73, 0x20, 0x61, 0x72, 0x65, 0x20, 0x65, 0x6E, 0x63, 0x6F, 0x64, 0x65, 0x64, 0x20,
	0x63, 0x6F, 0x72, 0x72, 0x65, 0x63, 0x74, 0x6C, 0x79, 0x20, 0x61, 0x66, 0x74, 0x65, 0x72, 0x20, 0x74, 0x68, 0x65,
	0x20, 0x6D, 0x61, 0x6A, 0x6F, 0x72, 0x2E, 0x20, 0x54, 0x68, 0x69, 0x73, 0x20, 0x6C, 0x69, 0x6E, 0x65, 0x20, 0x69,
	0x73, 0x20, 0x67, 0x72, 0x65, 0x61, 0x74, 0x65, 0x72, 0x20, 0x74, 0x68, 0x61, 0x6E, 0x20, 0x32, 0x35, 0x36, 0x20,
	0x63, 0x68, 0x61, 0x72, 0x61, 0x63, 0x74, 0x65, 0x72, 0x73, 0x20, 0x74, 0x6F, 0x20, 0x74, 0x65, 0x73, 0x74, 0x20,
	0x69, 0x66, 0x20, 0x6C, 0x65, 0x6E, 0x67, 0x74, 0x68, 0x73, 0x20, 0x61, 0x72, 0x65, 0x20, 0x65, 0x6E, 0x63, 0x6F,
	0x64, 0x65, 0x64, 0x20, 0x63, 0x6F, 0x72, 0x72, 0x65, 0x63, 0x74, 0x6C, 0x79, 0x20, 0x61, 0x66, 0x74, 0x65, 0x72,
	0x20, 0x74, 0x68, 0x65, 0x20, 0x6D, 0x61, 0x6A, 0x6F, 0x72, 0x2E, 0x68, 0x75, 0x6E, 0x73, 0x69, 0x67, 0x6E, 0x65,
	0x64, 0x08, 0x68, 0x6E, 0x65, 0x67, 0x61, 0x74, 0x69, 0x76, 0x65, 0x23, 0x19, 0x03, 0x09, 0x84, 0x0B, 0x35, 0xFB,
	0x40, 0x40, 0xA6, 0x66, 0x66, 0x66, 0x66, 0x66, 0x6B, 0x66, 0x6F, 0x75, 0x72, 0x74, 0x79, 0x2D, 0x66, 0x6F, 0x75,
	0x72,
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
	0xBF, 0x19, 0x02, 0x2B, 0xBF, 0x65, 0x66, 0x6C, 0x6F, 0x61, 0x74, 0xFB, 0x40, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
	0x00, 0x6A, 0x62, 0x79, 0x74, 0x65, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x45, 0x01, 0x02, 0x03, 0x04, 0x05, 0x6A,
	0x75, 0x74, 0x66, 0x38, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67, 0x7F, 0x6F, 0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD, 0xEF,
	0xBC, 0x8C, 0xE4, 0xB8, 0x96, 0xE7, 0x95, 0x8C, 0x63, 0x20, 0x2D, 0x20, 0x6C, 0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x2C,
	0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0xFF, 0x68, 0x75, 0x6E, 0x73, 0x69, 0x67, 0x6E, 0x65, 0x64, 0x08, 0x68, 0x6E,
	0x65, 0x67, 0x61, 0x74, 0x69, 0x76, 0x65, 0x23, 0xFF, 0x19, 0x03, 0x09, 0x9F, 0x0B, 0x35, 0xFB, 0x40, 0x40, 0xA6,
	0x66, 0x66, 0x66, 0x66, 0x66, 0x6B, 0x66, 0x6F, 0x75, 0x72, 0x74, 0x79, 0x2D, 0x66, 0x6F, 0x75, 0x72, 0xFF, 0xFF,
];

#[test]
fn decode_test() {
	let arr: [&[u8]; 2] = [&TEST_DATA_DEFINITE, &TEST_DATA_INDEFINITE];
	for test_data in &arr {
		let v: Vec<u8> = test_data.to_vec();
		let data = cborg::decode(&v).unwrap();

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
	let utf8_key = "utf8string";
	let utf8_val = "你好，世界 - hello, world";
	let long_key = "long string";
	let long_val = "This line is greater than 256 characters to test if lengths are encoded correctly after the major. This line is greater than 256 characters to test if lengths are encoded correctly after the major. This line is greater than 256 characters to test if lengths are encoded correctly after the major.";

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
					key: Value::Utf8String(utf8_key.to_string()),
					val: Value::Utf8String(utf8_val.to_string()),
				},
				KeyVal {
					key: Value::Utf8String(long_key.to_string()),
					val: Value::Utf8String(long_val.to_string()),
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
			println!("mismatch at pos {}: {} : {}", i, TEST_DATA_DEFINITE[i], bytes[i]);
		}
	}

	assert_eq!(TEST_DATA_DEFINITE.to_vec(), bytes);

	let mut data = HashMap::new();
	let mut inner555 = HashMap::new();
	inner555.insert(utf8_key, utf8_val);
	inner555.insert("second", "two");

	let mut inner777 = HashMap::new();
	inner777.insert("third", "three");

	data.insert(555, inner555);
	data.insert(777, inner777);

	let cbor = data.to_value().encode();
	let decoded: HashMap<u32, HashMap<String, String>> = match cborg::decode_to(&cbor) {
		Ok(x) => match x {
			Some(x) => x,
			None => {
				println!("Could not unmarshal cbor1 into dictionary");
				panic!();
			}
		},
		Err(e) => {
			println!("Error decoding cbor1: {}", e);
			panic!();
		}
	};
	assert_eq!(decoded[&555][utf8_key], utf8_val);
	assert_eq!(decoded[&555]["second"], "two");
	assert_eq!(decoded[&777]["third"], "three");

	let cbor = cborg::encode_dyn(&data);
	let decoded: HashMap<u32, HashMap<String, String>> = cborg::decode_to(&cbor).unwrap().unwrap();
	assert_eq!(decoded[&555][utf8_key], utf8_val);
	assert_eq!(decoded[&555]["second"], "two");
	assert_eq!(decoded[&777]["third"], "three");

	let cbor = cborg::encode_ref(&data);
	let decoded: HashMap<u32, HashMap<String, String>> = cborg::decode_to(&cbor).unwrap().unwrap();
	assert_eq!(decoded[&555][utf8_key], utf8_val);
	assert_eq!(decoded[&555]["second"], "two");
	assert_eq!(decoded[&777]["third"], "three");

	let cbor = cborg::encode(data);
	let decoded: HashMap<u32, HashMap<String, String>> = cborg::decode_to(&cbor).unwrap().unwrap();
	assert_eq!(decoded[&555][utf8_key], utf8_val);
	assert_eq!(decoded[&555]["second"], "two");
	assert_eq!(decoded[&777]["third"], "three");
}

#[test]
fn display_test() {
	let data = cborg::decode_slice(&TEST_DATA_INDEFINITE).unwrap();
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
	let v = cborg::decode(TEST_DATA_DEFINITE.iter()).unwrap();
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

	let dict: HashMap<u64, HashMap<String, String>> = cborg::decode_to(TEST_DATA_DEFINITE.iter()).unwrap().unwrap();
	assert_eq!(1, dict.len());
	assert!(dict.contains_key(&555));
	let map2 = dict.get(&555).unwrap();
	assert_eq!(2, map2.len());
	let val = map2.get(utf8_key).unwrap();
	assert_eq!(utf8_val, val);
	assert!(map2.get(longstring).unwrap().len() > 256);

	let dict: BTreeMap<i64, Vec<i64>> = cborg::decode_to(TEST_DATA_DEFINITE.iter()).unwrap().unwrap();
	let arr = dict.get(&777).unwrap();
	assert_eq!(2, arr.len());
	assert_eq!(11, arr[0]);
	assert_eq!(-22, arr[1]);
}

# CBORG
A [CBOR](https://cbor.io/) parser for Rust.

Incomplete and built for my own use-case. You probably want [serde_cbor](https://crates.io/crates/serde_cbor)

## Usage
`decode_to` will decode CBOR and unmarshal it into a given object:
```rust
// Unmarshal Map
use std::collections::HashMap;
let bytes = &[0b1010_0010, 0b0011_1000, 0b0001_1000, 0b0110_0011, 0x61, 0x62, 0x63,
              0b0000_0111, 0b0110_0011, 0x44, 0x45, 0x46];
let map: HashMap<i8, String> = cborg::decode_to(bytes).unwrap().unwrap();
assert_eq!("abc", map[&-25]);
assert_eq!("DEF", map[&7]);
```
```rust
// Unmarshal Array
let bytes = &[0b1000_0011, 11, 22, 0b0001_1000, 33];
let array: Vec<u32> = cborg::decode_to(bytes).unwrap().unwrap();
assert_eq!(11, array[0]);
assert_eq!(22, array[1]);
assert_eq!(33, array[2]);
```

[![pipeline status](https://gitlab.com/travbid/cborg/badges/master/pipeline.svg)](https://gitlab.com/travbid/cborg/commits/master)

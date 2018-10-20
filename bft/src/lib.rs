#![feature(custom_attribute)]
#![feature(nll)]
#![feature(vec_remove_item)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate rmp;
extern crate rmp_serde as rmps;
#[macro_use]
extern crate runtime_fmt;

extern crate bigint;
extern crate rand;
extern crate chrono;
extern crate hex;
extern crate sha3;
extern crate rlp;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate ethereum_types;
extern crate secp256k1;
#[macro_use]
extern crate cryptocurrency_kit;
extern crate lru_time_cache;

pub mod common;
pub mod consensus;
pub mod types;

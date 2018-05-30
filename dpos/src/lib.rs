// spell-checker:ignore cors

#![deny(missing_debug_implementations, unsafe_code)]
#![cfg_attr(feature = "flame_profile", feature(plugin, custom_attribute))]
#![cfg_attr(feature = "flame_profile", plugin(exonum_flamer))]
#![cfg_attr(feature = "long_benchmarks", feature(test))]

pub mod prelude {

}

extern crate time;
extern crate atty;
extern crate bit_vec;
extern crate bodyparser;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
#[macro_use(crate_version, crate_authors)]
extern crate clap;
extern crate colored;
extern crate env_logger;
extern crate exonum_rocksdb as rocksdb;
extern crate exonum_sodiumoxide as sodiumoxide;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate hex;
#[macro_use]
extern crate log;
extern crate os_info;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate term;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_retry;
#[cfg(any(test, feature = "long_benchmarks"))]
extern crate tokio_timer;
extern crate toml;
extern crate uuid;
extern crate vec_map;

// Test dependencies.
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
extern crate tempdir;
#[cfg(all(test, feature = "long_benchmarks"))]
extern crate test;

#[macro_use]
pub mod encoding;
#[macro_use]
pub mod messages;
#[macro_use]
pub mod helpers;
pub mod crypto;
pub mod blockchain;
pub mod storage;
pub mod consensue;
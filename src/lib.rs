#![feature(custom_attribute)]
#![feature(nll)]
#![feature(vec_remove_item)]
#![feature(get_type_id)]
#![feature(duration_as_u128)]
#![feature(await_macro, futures_api, async_await)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde_millis;
#[macro_use]
extern crate runtime_fmt;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate ethereum_types;
#[macro_use]
extern crate cryptocurrency_kit;
#[macro_use]
extern crate actix;
extern crate actix_broker;
#[macro_use]
extern crate crossbeam;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate failure;

pub mod common;
pub mod util;
pub mod consensus;
pub mod types;
pub mod store;
pub mod core;
pub mod protocol;
pub mod p2p;
pub mod error;
pub mod pprof;
#[macro_use]
pub mod subscriber;
pub mod minner;
pub mod cmd;
pub mod config;
pub mod logger;
pub mod mocks;
pub mod api;
use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::ethkey::Address;

use super::{Bloom, Difficulty, Gas, Height, Timestamp};

pub struct Header {
    parent_hash: Hash,
    coin_base: Address,
    root: Hash,
    tx_hash: Hash,
    receipt_hash: Hash,
    bloom: Bloom,
    difficulty: Difficulty,
    height: Height,
    gas_limit: Gas,
    gas_used: Gas,
    time: Timestamp,
    extra: Vec<u8>,
}

pub struct Block {
    header: Option<Header>,
}

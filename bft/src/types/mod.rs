use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::common::to_hex;

use std::clone::Clone;
use std::cmp::{Ord, Ordering, PartialEq};
use std::fmt::Display;

pub mod transaction;
pub mod block;
pub mod votes;

lazy_static! {
    pub static ref EMPTY_ADDRESS: Address = {
        Address::from(0)
    };
}

pub type Height = u64;
pub type Timestamp = u64;
pub type Bloom = u64;
pub type Difficulty = u64;
pub type Gas = u64;

#[derive(Debug, Clone, Eq)]
pub struct Validator {
    address: Address,
}

impl Ord for Validator {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}

impl PartialOrd for Validator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Validator {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Display for Validator {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", to_hex(self.address))
    }
}

impl Validator {
    pub fn new(address: Address) -> Self {
        Validator { address }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }
}

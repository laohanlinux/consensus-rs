use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::common::to_hex;
use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash, EMPTY_HASH};
use cryptocurrency_kit::storage::values::StorageValue;
use serde::{Deserialize, Serialize};

use std::io::Cursor;
use std::borrow::Cow;
use std::collections::HashMap;

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

pub type Validators = Vec<Validator>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorArray {
    inner: Vec<Address>,
    index: HashMap<Address, usize>,
}

implement_cryptohash_traits! {ValidatorArray}
implement_storagevalue_traits! {ValidatorArray}

impl ValidatorArray {
    pub fn new(addresses: Vec<Address>) -> ValidatorArray {
        let mut index: HashMap<Address, usize> = HashMap::new();
        addresses.iter().fold(0, |acc, address| {
            index.insert(*address, acc as usize);
            acc + 1
        });
        ValidatorArray {
            inner: addresses,
            index: index,
        }
    }

    pub fn have(&self, address: &Address) -> bool {
        self.index.contains_key(address)
    }
}

impl From<Vec<Validator>> for ValidatorArray {
    fn from(validators: Vec<Validator>) -> ValidatorArray {
        let addresses = validators.iter().map(|validator| { validator.address }).collect();
        ValidatorArray::new(addresses)
    }
}

impl From<Vec<Address>> for ValidatorArray {
    fn from(addresses: Vec<Address>) -> ValidatorArray {
        ValidatorArray::new(addresses)
    }
}

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct Validator {
    address: Address,
}

implement_cryptohash_traits! {Validator}
implement_storagevalue_traits! {Validator}

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

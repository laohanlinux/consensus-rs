use cryptocurrency_kit::common::to_hex;
use cryptocurrency_kit::ethkey::Address;

use std::cmp::{Ord, Ordering, PartialEq};

trait Validator: ::std::fmt::Debug + ::std::fmt::Display {
    fn address(&self) -> Address;
}

type Validators = Vec<Box<dyn Validator>>;

#[derive(Debug, Eq)]
struct ImplValidator {
    address: Address,
}

impl Ord for ImplValidator {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}

impl PartialOrd for ImplValidator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ImplValidator {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Validator for ImplValidator {
    fn address(&self) -> Address {
        self.address
    }
}

impl ::std::fmt::Display for ImplValidator {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", to_hex(self.address))
    }
}

trait ValidatorSet: Clone {
    fn calc_proposer(last_proposer: Address, round: u64);
    fn size(&self) -> usize;
    fn list(&self) -> Vec<Box<dyn Validator>>;
    fn get_by_index(&self) -> Validator;
    fn get_by_address(&self, address: Address) -> Validator;
    // get current proposer
    fn get_proposer(&self) -> Validator;
    fn is_proposer(&self, address: Address) -> bool;
    fn add_validator(&mut self, address: Address) -> bool;
    fn remove_validator(&mut self, address: Address) -> bool;
    fn f() -> isize;
}

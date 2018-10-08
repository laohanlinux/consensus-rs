use cryptocurrency_kit::common::to_hex;
use cryptocurrency_kit::ethkey::Address;

use super::types::Height;
use std::cmp::{Ord, Ordering, PartialEq};

pub trait Validator: ::std::fmt::Debug + ::std::fmt::Display {
    fn address(&self) -> Address;
}

pub type Validators = Vec<Box<dyn Validator>>;

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

pub trait ValidatorSet: Clone {
    fn calc_proposer(last_proposer: Address, round: u64);
    fn size(&self) -> usize;
    fn list(&self) -> Vec<Box<Validator>>;
    fn get_by_index<T: Validator>(&self) -> Option<T>;
    fn get_by_address(&self, address: Address) -> Option<Box<Validator>>;
    // get current proposer
    fn get_proposer(&self) -> &Validator;
    fn is_proposer(&self, address: Address) -> bool;
    fn add_validator(&mut self, address: Address) -> bool;
    fn remove_validator(&mut self, address: Address) -> bool;
    fn f() -> isize;
}

type ProposalSelector = Box<Fn()>;

fn selectorFn() {}

struct ImplValidatorSet<T: Validator>{
    validators: Validators,
    proposer: T,
    selector: ProposalSelector,
}


impl<T: Validator> ImplValidatorSet<T> {
//    fn new<T>(address: &[Address]) -> Self {
//        let mut set = ImplValidatorSet{
//            validators: Vec::new();
//            proposer: Box::new(selectorFn),
//        };
//
//        let mut validators:Vec<Address> = Vec::new();
//        for x in address {
//            validators.push(x.clone());
//        }
//
//        // TODO sort address
//
//
//    }

    pub fn size(&self) -> usize {
        self.validators.len()
    }

    pub fn list(&self) -> Box<Validators> {
        let mut validators = Vec::new();
        for validator in self.validators {
            validators.push(validator);
        }
        Box::new(validators)
    }
}
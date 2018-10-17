use cryptocurrency_kit::common::to_hex;
use cryptocurrency_kit::ethkey::Address;

use super::types::Height;
use std::cmp::{Ord, Ordering, PartialEq};

pub trait Validator: ::std::clone::Clone + ::std::fmt::Debug + ::std::fmt::Display {
    fn address(&self) -> Address;
}

pub type Validators<T: Validator> = Vec<T>;

#[derive(Debug, Clone, Eq)]
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

pub trait ValidatorSet<T: Validator>: Clone {
    fn calc_proposer(&self, last_proposer: Address, round: u64);
    fn size(&self) -> usize;
    fn list(&self) -> Validators<T>;
    fn get_by_index(&self) -> Option<T>;
    fn get_by_address(&self, address: Address) -> Option<T>;
    // get current proposer
    fn get_proposer(&self) -> &T;
    fn is_proposer(&self, address: Address) -> bool;
    fn add_validator(&mut self, address: Address) -> bool;
    fn remove_validator(&mut self, address: Address) -> bool;
    fn f(&self) -> isize;
}

type ProposalSelector<T> = fn(height: Height, vals: Validators<T>) -> T;

fn selectorFn<T: Validator>(height: Height, vals: Validators<T>) -> T {
    let idx = vals.len() % height as usize;
    vals[idx].clone()
}

struct ImplValidatorSet<T: Validator> {
    validators: Validators<T>,
    proposer: Option<T>,
    selector: Box<ProposalSelector<T>>,
}

impl<T: Validator> ImplValidatorSet<T> {
    fn new(address: &[Address]) -> Self {
        let mut set = ImplValidatorSet {
            validators: Vec::new(),
            proposer: None,
            selector: Box::new(selectorFn),
        };

        let mut validators: Vec<Address> = Vec::new();
        for x in address {
            validators.push(x.clone());
        }

        set.validators = validators;
        set
    }

    pub fn size(&self) -> usize {
        self.validators.len()
    }
}

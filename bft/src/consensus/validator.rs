use cryptocurrency_kit::common::to_hex;
use cryptocurrency_kit::ethkey::Address;

use super::types::Height;

use std::cmp::{Ord, Ordering, PartialEq};
use std::clone::Clone;
use std::fmt::{Debug, Display};

pub trait Validator: Clone + Debug + Display {
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

impl Display for ImplValidator {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{}", to_hex(self.address))
    }
}

impl ImplValidator {
    pub fn new(address: Address) -> Self {
        ImplValidator{address}
    }
}

pub trait ValidatorSet<T: Validator>: Clone {
    fn calc_proposer(&mut self, last_proposer: Address, round: u64);
    fn size(&self) -> usize;
    fn list(&self) -> Validators<T>;
    fn get_by_index(&self, index: usize) -> Option<&T>;
    fn get_by_address(&self, address: Address) -> Option<&T>;
    // get current proposer
    fn get_proposer(&self) -> Option<&T>;
    fn is_proposer(&self, address: Address) -> bool;
    fn add_validator(&mut self, address: Address) -> bool;
    fn remove_validator(&mut self, address: Address) -> bool;
    fn f(&self) -> isize;
}

type ProposalSelector<T> = fn(height: Height, vals: Validators<T>) -> T;

fn fn_selector<T: Validator>(height: Height, vals: Validators<T>) -> T {
    let idx = vals.len() % height as usize;
    vals[idx].clone()
}

#[derive(Clone)]
struct ImplValidatorSet {
    validators: Validators<ImplValidator>,
    proposer: Option<ImplValidator>,
    selector: Box<ProposalSelector<ImplValidator>>,
}

impl ImplValidatorSet {
    pub fn new(address: &[Address]) -> ImplValidatorSet {
        let mut set = ImplValidatorSet {
            validators: Vec::new(),
            proposer: None,
            selector: Box::new(fn_selector),
        };

        for x in address {
            set.validators.push(ImplValidator::new(x.clone()));
        }
        set.validators.sort_by_key(|k| k.address);
        set
    }
}

impl ValidatorSet<ImplValidator> for ImplValidatorSet {
    fn calc_proposer(&mut self, last_proposer: Address, round: u64) {

    }

    fn size(&self) -> usize {
        self.validators.len()
    }

    // TODO Optz, advoid copy
    fn list(&self) -> Validators<ImplValidator> {
        self.validators.clone()
    }

    fn get_by_index(&self, index: usize) -> Option<&ImplValidator> {
        self.validators.get(index).to_owned()
    }

    fn get_by_address(&self, address: Address) -> Option<&ImplValidator> {
        let mut idx = 0;
        let ok = self.validators.iter().any(|validator| {
            if validator.address == address {
                return true;
            }
            idx += 1;
            false
        });
        if !ok {
            return None;
        }
        self.validators.get(idx).to_owned()
    }

    fn get_proposer(&self) -> Option<&ImplValidator> {
        self.proposer.as_ref()
    }

    fn is_proposer(&self, address: Address) -> bool {
        if self.proposer.is_none() {
            return false;
        }
        match self.proposer {
            None => false,
            Some(ref proposer) => proposer.address == address,
        }
    }

    fn add_validator(&mut self, address: Address) -> bool {
        if self.validators.iter().any(|validator| validator.address == address) {
            return false;
        }
        self.validators.push(ImplValidator::new(address));
        self.validators.sort_by_key(|validator|validator.address);
        true
    }

    fn remove_validator(&mut self, address: Address) -> bool {
        match self.validators.remove_item(&ImplValidator::new(address)) {
            None => false,
            _ => true,
        }
    }

    // TODO
    fn f(&self) -> isize {
        0
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{self, Write};

    #[test]
    fn test_address_sort() {
        let mut address_list = vec![Address::from(100),
                                    Address::from(10),
                                    Address::from(21),
                                    Address::from(31),
                                    Address::from(3)];
        let expect_address_list = vec![Address::from(3),
                                       Address::from(10),
                                       Address::from(21),
                                       Address::from(31),
                                       Address::from(100)];

        let val_set = ImplValidatorSet::new(&address_list);
        val_set.validators.iter().fold(0, |acc, x|{
            assert_eq!(x.address, expect_address_list[acc]);
            acc + 1
        });

        assert_eq!(val_set.size(), expect_address_list.len());
    }

    #[test]
    fn tets_validator_set() {
        let mut address_list = vec![Address::from(100),
                                    Address::from(10),
                                    Address::from(21),
                                    Address::from(31),
                                    Address::from(3)];
        let val_set = ImplValidatorSet::new(&address_list);
    }
}
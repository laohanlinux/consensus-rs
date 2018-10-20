use cryptocurrency_kit::common::to_hex;
use cryptocurrency_kit::ethkey::Address;

use super::types::Height;

use std::clone::Clone;
use std::cmp::{Ord, Ordering, PartialEq};
use std::fmt::{Debug, Display};

pub type Validators = Vec<Validator>;

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
    pub fn address(&self) -> Address {
        self.address
    }

    pub fn new(address: Address) -> Self {
        Validator { address }
    }
}

pub trait ValidatorSet {
    fn calc_proposer(&mut self, last_proposer: Address, round: u64);
    fn size(&self) -> usize;
    fn list(&self) -> Validators;
    fn get_by_index(&self, index: usize) -> Option<&Validator>;
    fn get_by_address(&self, address: Address) -> Option<&Validator>;
    // get current proposer
    fn get_proposer(&self) -> Option<&Validator>;
    fn is_proposer(&self, address: Address) -> bool;
    fn add_validator(&mut self, address: Address) -> bool;
    fn remove_validator(&mut self, address: Address) -> bool;
    fn f(&self) -> isize;
}

type ProposalSelector = fn(height: Height, vals: Validators) -> Validator;

fn fn_selector(height: Height, vals: Validators) -> Validator {
    let idx = vals.len() % height as usize;
    vals[idx].clone()
}

#[derive(Clone)]
pub struct ImplValidatorSet {
    validators: Validators,
    proposer: Option<Validator>,
    selector: Box<ProposalSelector>,
}

impl ImplValidatorSet {
    pub fn new(address: &[Address]) -> ImplValidatorSet {
        let mut set = ImplValidatorSet {
            validators: Vec::new(),
            proposer: None,
            selector: Box::new(fn_selector),
        };

        for x in address {
            set.validators.push(Validator::new(x.clone()));
        }
        set.validators.sort_by_key(|k| k.address);
        set
    }
}

impl ValidatorSet for ImplValidatorSet {
    fn calc_proposer(&mut self, last_proposer: Address, round: u64) {
        let mut idx = 0;
        let ok = self.validators.iter().any(|validator| {
            if validator.address == last_proposer {
                return true;
            }
            idx += 1;
            false
        });
        assert!(ok);

        let next_proposer_local = (idx + round) as usize % self.validators.len();
        match self.validators.get(next_proposer_local) {
            None => unreachable!(),
            Some(validator) => {
                self.proposer = Some(validator.clone());
            }
        }
    }

    fn size(&self) -> usize {
        self.validators.len()
    }

    // TODO Optz, advoid copy
    fn list(&self) -> Validators {
        self.validators.clone()
    }

    fn get_by_index(&self, index: usize) -> Option<&Validator> {
        self.validators.get(index).to_owned()
    }

    fn get_by_address(&self, address: Address) -> Option<&Validator> {
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

    fn get_proposer(&self) -> Option<&Validator> {
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
        if self
            .validators
            .iter()
            .any(|validator| validator.address == address)
        {
            return false;
        }
        self.validators.push(Validator::new(address));
        self.validators.sort_by_key(|validator| validator.address);
        true
    }

    fn remove_validator(&mut self, address: Address) -> bool {
        match self.validators.remove_item(&Validator::new(address)) {
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
        let mut address_list = vec![
            Address::from(100),
            Address::from(10),
            Address::from(21),
            Address::from(31),
            Address::from(3),
        ];
        let expect_address_list = vec![
            Address::from(3),
            Address::from(10),
            Address::from(21),
            Address::from(31),
            Address::from(100),
        ];

        let val_set = ImplValidatorSet::new(&address_list);
        val_set.validators.iter().fold(0, |acc, x| {
            assert_eq!(x.address, expect_address_list[acc]);
            acc + 1
        });

        assert_eq!(val_set.size(), expect_address_list.len());
    }

    #[test]
    fn tets_validator_set() {
        let mut address_list = vec![
            Address::from(100),
            Address::from(10),
            Address::from(21),
            Address::from(31),
            Address::from(3),
        ];
        let val_set = ImplValidatorSet::new(&address_list);
    }
}

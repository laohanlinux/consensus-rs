use cryptocurrency_kit::common::to_hex;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::Hash;
use bigint::U128;

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
    fn calc_proposer(&mut self, prex_blh: &Hash, height: Height, round: u64);
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

// blh: parent block hash
// height: current height
// round: current round
// vals: current validator's set
/// TODO Opitz
type ProposalSelector = fn(blh: & Hash, height: Height, round: u64, vals: & Validators) -> Validator;

fn fn_selector(blh: & Hash, height: Height, round: u64, vals: & Validators) -> Validator {
    assert!(vals.len() > 0);
    let seed = (randon_seed(blh, height, vals) + round) % vals.len() as u64;
    vals[seed as usize].clone()
}

fn randon_seed(blh: &Hash, height: Height, vals: &Validators) -> u64 {
    let blh = blh.as_ref();
    let mut seed_buf = [0; 16];
    for (idx, item) in seed_buf[..8].iter_mut().enumerate() {
        *item = blh[idx];
    }

    let block_seed: U128 = U128::from(seed_buf);
    (block_seed % U128::from(vals.len())).as_u64()
}

#[derive(Clone)]
pub struct ImplValidatorSet {
    validators: Validators,
    proposer: Option<Validator>,
    selector: Box<ProposalSelector>,
}

impl ImplValidatorSet {
    pub fn new(address: &[Address], selector: Box<ProposalSelector>) -> ImplValidatorSet {
        let mut set = ImplValidatorSet {
            validators: Vec::new(),
            proposer: None,
            selector,
        };

        for x in address {
            set.validators.push(Validator::new(x.clone()));
        }
        set.validators.sort_by_key(|k| k.address);
        set
    }
}

impl ValidatorSet for ImplValidatorSet {
    fn calc_proposer(&mut self, pre_blh: &Hash, height: Height, round: u64) {
        let next_proposer = (self.selector)(pre_blh, height, round, &self.validators);
        self.proposer = Some(next_proposer);
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
    use cryptocurrency_kit::crypto::HASH_SIZE;
    use rand::prelude::*;

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

        let val_set = ImplValidatorSet::new(&address_list, Box::new(fn_selector));
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
        let mut val_set = ImplValidatorSet::new(&address_list, Box::new(fn_selector));
        {
            (0..address_list.len() * 3).for_each(|round|{
                val_set.calc_proposer(&Hash::zero(), 0, round as u64);
                writeln!(io::stdout(), "round:{}, proposer: {}", round, val_set.proposer.as_ref().unwrap()).unwrap();
            })
        }

        writeln!(io::stdout(), "========================").unwrap();
        {
            let mut random_hash = || {
                let mut hash = [0; HASH_SIZE];
                (0..HASH_SIZE).for_each(|bit|{
                    let x: u8 = random();
                    hash[bit] = x;
                });
                Hash::new(hash.as_ref())
            };

            (0 ..address_list.len() * 3).for_each(|_|{
                let hash = random_hash();
                val_set.calc_proposer( &hash, 0, 0);
                writeln!(io::stdout(), "round:{}, proposer: {}", 0, val_set.proposer.as_ref().unwrap()).unwrap();
            })
        }
    }
}

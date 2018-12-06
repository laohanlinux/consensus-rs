use bigint::U128;
use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::ethkey::Address;
use ethereum_types::H160;

use types::{Height, Validator};

pub type Validators = Vec<Validator>;

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
    fn fault(&self) -> usize;
    fn two_thirds_majority(&self) -> usize;
    fn has_two_thirds_majority(&self, n: usize) -> bool;
}

// blh: parent block hash
// height: current height
// round: current round
// vals: current validator's set
/// TODO Opitz
type ProposalSelector = fn(blh: &Hash, height: Height, round: u64, vals: &Validators) -> Validator;

pub fn fn_selector(blh: &Hash, height: Height, round: u64, vals: &Validators) -> Validator {
    assert!(!vals.is_empty());
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
            set.validators.push(Validator::new(*x));
        }
        set.validators.sort_by_key(|k| *k.address());
        set
    }
}

impl ValidatorSet for ImplValidatorSet {
    fn calc_proposer(&mut self, pre_blh: &Hash, pre_height: Height, round: u64) {
        let next_proposer = (self.selector)(pre_blh, pre_height, round, &self.validators);
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
            if *validator.address() == address {
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
            Some(ref proposer) => *proposer.address() == address,
        }
    }

    fn add_validator(&mut self, address: Address) -> bool {
        if self
            .validators
            .iter()
            .any(|validator| *validator.address() == address)
            {
                return false;
            }
        self.validators.push(Validator::new(address));
        self.validators
            .sort_by_key(|validator| *validator.address());
        true
    }

    fn remove_validator(&mut self, address: Address) -> bool {
        match self.validators.remove_item(&Validator::new(address)) {
            None => false,
            _ => true,
        }
    }

    fn fault(&self) -> usize {
        let vals_size = self.validators.len() as f32;
        let ceil = vals_size * 1.0 / 3.0;
        ceil.ceil() as usize
    }

    // TODO
    fn two_thirds_majority(&self) -> usize {
        let vals_size = self.validators.len() as f32;
        let ceil = vals_size * 2.0 / 3.0;
        ceil.ceil() as usize
    }

    fn has_two_thirds_majority(&self, n: usize) -> bool {
        n >= self.two_thirds_majority()
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
            assert_eq!(*x.address(), expect_address_list[acc]);
            acc + 1
        });

        assert_eq!(val_set.size(), expect_address_list.len());
    }

    #[test]
    fn test_validator_set() {
        let mut address_list = vec![
            Address::from(100),
            Address::from(10),
            Address::from(21),
            Address::from(31),
            Address::from(3),
        ];
        let mut val_set = ImplValidatorSet::new(&address_list, Box::new(fn_selector));

        /// size
        assert_eq!(address_list.len(), val_set.size());

        /// list and get_by_address
        {
            let address_list = val_set.list();
            address_list.iter().for_each(|validator| {
                assert!(val_set.get_by_address(*validator.address()).is_some());
            });
        }

        /// get_by_index
        (0..val_set.size()).for_each(|idx| {
            assert!(val_set.get_by_index(idx).is_some());
        });

        /// get_proposer(&self)
        {
            assert!(val_set.get_proposer().is_none());
            val_set.calc_proposer(&Hash::zero(), 0, 0);
            assert!(val_set.get_proposer().is_some());
        }

        // is_proposer(&self, address: Address) -> bool;
        {
            let proposer = val_set.get_proposer().unwrap();
            assert!(val_set.is_proposer(*proposer.address()));
        }

        /// add new validator
        {
            let new_address = Address::from(random::<u64>());
            assert!(val_set.add_validator(new_address));
            assert!(val_set.get_by_address(new_address).is_some());
            // readd same proposer
            assert!(val_set.add_validator(new_address) == false);
        }

        /// remove a validator
        {
            let remove_validator = Validator::new(address_list[0]);
            assert!(val_set.remove_validator(*remove_validator.address()));
            // remove again same validator
            assert!(val_set.remove_validator(*remove_validator.address()) == false);
        }

        // calc_proposer
        {
            (0..address_list.len() * 3).for_each(|round| {
                val_set.calc_proposer(&Hash::zero(), 0, round as u64);
                writeln!(
                    io::stdout(),
                    "round:{}, proposer: {}",
                    round,
                    val_set.proposer.as_ref().unwrap()
                )
                    .unwrap();
            })
        }
        writeln!(io::stdout(), "========================").unwrap();
        {
            let mut random_hash = || {
                let mut hash = [0; HASH_SIZE];
                (0..HASH_SIZE).for_each(|bit| {
                    let x: u8 = random();
                    hash[bit] = x;
                });
                Hash::new(hash.as_ref())
            };

            (0..address_list.len() * 3).for_each(|_| {
                let hash = random_hash();
                val_set.calc_proposer(&hash, 0, 0);
                writeln!(
                    io::stdout(),
                    "round:{}, proposer: {}",
                    0,
                    val_set.proposer.as_ref().unwrap()
                )
                    .unwrap();
            })
        }
    }

    #[test]
    fn test_validator_set_two_third() {
        /// more than 3 validators
        {
            let mut address_list = vec![
                Address::from(100),
                Address::from(10),
                Address::from(21),
                Address::from(31),
                Address::from(3),
            ];
            let mut val_set = ImplValidatorSet::new(&address_list, Box::new(fn_selector));
            assert_eq!(val_set.two_thirds_majority(), 4);
            assert!(val_set.has_two_thirds_majority(4));
            assert!(!val_set.has_two_thirds_majority(3));
            writeln!(io::stdout(), "+2/3=> {}", val_set.two_thirds_majority()).unwrap();
        }

        /// equal 3 validators
        {
            let mut address_list = vec![Address::from(100), Address::from(10), Address::from(3)];
            let mut val_set = ImplValidatorSet::new(&address_list, Box::new(fn_selector));
            assert_eq!(val_set.two_thirds_majority(), 3);
            assert!(val_set.has_two_thirds_majority(4));
            assert!(!val_set.has_two_thirds_majority(2));
            writeln!(io::stdout(), "+2/3=> {}", val_set.two_thirds_majority()).unwrap();
        }
    }
}

use std::fmt::{self, LowerHex, Formatter};
use std::sync::Arc;

use ethereum_types::U256;
use cryptocurrency_kit::{
    ethkey::Address,
    crypto::{CryptoHash, Hash, hash},
};
use transaction_pool::VerifiedTransaction;


use crate::{
    types::{Gas, Height, Timestamp},
    types::transaction::Transaction,
};

#[derive(Debug, Default, Clone)]
pub struct TransactionBuilder {
    nonce: U256,
    gas_price: Gas,
    gas: Gas,
    sender: Address,
    mem_usage: usize,
}

impl TransactionBuilder {
    pub fn tx(&self) -> Self {
        self.clone()
    }

    pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
        self.nonce = nonce.into();
        self
    }

    pub fn gas_price<T: Into<Gas>>(mut self, gas_price: T) -> Self {
        self.gas_price = gas_price.into();
        self
    }

    pub fn sender<T: Into<Address>>(mut self, sender: T) -> Self {
        self.sender = sender.into();
        self
    }

    pub fn mem_usage(mut self, mem_usage: usize) -> Self {
        self.mem_usage = mem_usage;
        self
    }

//    pub fn new(self) -> Transaction {
//        let hash = self.nonce ^ (U256::from(100) * self.gas_price) ^ (U256::from(100_000) * U256::from(self.sender.low_u64()));
////        Transaction::new()
//    }
}

impl VerifiedTransaction for Transaction {
    type Hash = Hash;
    type Sender = Address;
    fn hash(&self) -> &Hash { self.get_hash().as_ref().unwrap() }

    /// TODO
    fn mem_usage(&self) -> usize { 0 }

    fn sender(&self) -> &Address { self.to().unwrap() }
}

pub type SharedTransaction =  Arc<Transaction>;

#[cfg(test)]
mod tests {
    use super::*;
    use transaction_pool::Pool;
    

    #[test]
    fn t_transaction_pool(){

    }
}
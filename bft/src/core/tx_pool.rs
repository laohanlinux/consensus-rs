use std::sync::Arc;
use std::collections::BTreeMap;

use actix::prelude::*;
use priority_queue::PriorityQueue;
use cryptocurrency_kit::crypto::{Hash, hash, EMPTY_HASH};
use evmap::{self, WriteHandle, ReadHandle};

use crate::{
    types::transaction::Transaction,
    error::TxPoolError,
};

pub const MAX_TXPOOL_SIZE: u64 = 10_000_000;
pub const MAX_SLOT_SIZE: u32 = 1_000;

pub trait TxPool {
    fn get_tx(&self, tx_hash: &Hash) -> Option<&Transaction>;
    fn get_n_tx(&self, n: u64) -> Vec<&Transaction>;
    fn add_tx(&mut self, transaction: Transaction) -> Result<u64, TxPoolError>;
    fn add_txs(&mut self, transactions: &Vec<Transaction>) -> Result<u64, TxPoolError>;
    fn remove_txs(&mut self, tx_hashes: Vec<&Hash>);
}

pub struct BaseTxPool {
    pq: PriorityQueue<Hash, u64>,
    txs: Vec<BTreeMap<Hash, Transaction>>,
}

impl Actor for BaseTxPool {
    type Context = Context<Self>;
}

impl TxPool for BaseTxPool {
    fn get_tx(&self, tx_hash: &Hash) -> Option<&Transaction> {
        self.txs[self.get_idx(tx_hash)].get(tx_hash)
    }

    fn get_n_tx(&self, n: u64) -> Vec<&Transaction> {
        let mut txs = vec![];
        let mut i: u64 = 0;
        for (tx_hash, _) in self.pq.iter() {
            if i >= n {
                break;
            }
            let idx = self.get_idx(tx_hash);
            let m = self.txs.get(idx).unwrap();
            txs.push(m.get(&tx_hash).unwrap());
        }
        txs
    }

    fn add_tx(&mut self, tx: Transaction) -> Result<u64, TxPoolError> {
        let idx = self.get_idx(tx.get_hash().unwrap());
        let mut v: &mut BTreeMap<_, _> = self.txs.get_mut(idx).unwrap();
        if v.get(&tx.get_hash().unwrap()).is_some() {
            return Ok(self.pq.len() as u64);
        }
        v.insert(tx.get_hash().unwrap().clone(), tx.clone());
        self.pq.push(tx.get_hash().unwrap().clone(), tx.amount());
        Ok(self.pq.len() as u64)
    }

    fn add_txs(&mut self, txs: &Vec<Transaction>) -> Result<u64, TxPoolError> {
        let mut start: u64 = 0;
        for tx in txs {
            self.add_tx(tx.clone())?;
            start += 1;
        }
        Ok(start)
    }

    fn remove_txs(&mut self, tx_hashes: Vec<&Hash>) {
        tx_hashes.iter().for_each(|tx_hash| {
            let idx = self.get_idx(tx_hash);
            let m: &mut BTreeMap<_, _> = self.txs.get_mut(idx).unwrap();
            m.remove(tx_hash);
        });
    }
}

impl BaseTxPool {
    pub fn new() -> Self {
        let n = (MAX_TXPOOL_SIZE / u64::from(MAX_SLOT_SIZE)) as usize;
        let mut tx_pool = BaseTxPool {
            pq: PriorityQueue::new(),
            txs: Vec::with_capacity(n),
        };
        (0..n).for_each(|_| {
            tx_pool.txs.push(BTreeMap::new());
        });
        tx_pool
    }
    fn get_idx(&self, tx_hash: &Hash) -> usize {
        use ethereum_types::U256;
        let u = U256::from(tx_hash.as_ref());
        (u % U256::from(self.txs.len())).as_u64() as usize
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;

    struct TxPoolActor {
        tx_pool: Arc<RwLock<Box<TxPool>>>,
    }

    impl Actor for TxPoolActor {
        type Context = Context<Self>;
    }

    #[test]
    fn t_txpool() {
//        let mut v = vec![];
        (0..10_0000).for_each(|idx| {})
    }
}
use std::sync::Arc;

use crossbeam::scope;
use ::actix::prelude::*;
use actix_broker::{BrokerSubscribe, BrokerIssue};
use parking_lot::RwLock;
use crossbeam::{Sender, Receiver, channel::bounded};
use rand::random;
use cryptocurrency_kit::ethkey::{Address, KeyPair};
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::crypto::CryptoHash;
use cryptocurrency_kit::crypto::hash;
use tokio_threadpool;
use futures::*;
use futures::sync::oneshot;

use crate::{
    subscriber::events::ChainEvent,
    core::chain::Chain,
    core::tx_pool::{TxPool, SafeTxPool},
    consensus::consensus::{Engine, SafeEngine},
    types::Timestamp,
    types::block::{Block, Header},
    types::transaction::{Transaction, merkle_root_transactions},
};

pub struct Minner {
    minter: Address,
    key_pair: KeyPair,
    chain: Arc<Chain>,
    txpool: Arc<RwLock<SafeTxPool>>,
    engine: Box<Engine>,
    seal_tx: Sender<()>,
    seal_rx: Receiver<()>,
    worker: tokio_threadpool::ThreadPool,
}

impl Actor for Minner {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_async::<ChainEvent>(ctx);
        info!("Start minner actor");
        self.mine(self.seal_rx.clone());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Minner actor has stoppped");
    }
}


impl Handler<ChainEvent> for Minner {
    type Result = ();
    fn handle(&mut self, msg: ChainEvent, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ChainEvent::NewHeader(last_header) => {
                debug!("Receive a new header event notify, hash:{:?}, height: {:?}", last_header.block_hash(), last_header.height);
                // stop current consensus
                self.seal_tx.send(()).unwrap();
                let seal = self.seal_rx.clone();
                self.mine(seal);
            }
            _ => {}
        }
    }
}

impl Minner {
    pub fn new(minter: Address,
               key_pair: KeyPair,
               chain: Arc<Chain>,
               txpool: Arc<RwLock<SafeTxPool>>,
               engine: SafeEngine,
               tx: Sender<()>,
               rx: Receiver<()>) -> Self {
        Minner {
            minter,
            key_pair,
            chain,
            txpool,
            engine,
            seal_tx: tx,
            seal_rx: rx,
            worker: tokio_threadpool::ThreadPool::new(),
        }
    }

    fn mine(&mut self, abort: Receiver<()>) {
        debug!("Ready to mine next block");
        let mut block = self.packet_next_block();
        match self.engine.seal(&mut block, abort) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to seal consensus, err: {:?}", err);
            }
        }
    }

    fn packet_next_block(&self) -> Block {
        let (next_time, pre_header) = self.next_block();
        let coinbase = self.coinbase_transaction();

        let pre_hash: Hash = pre_header.block_hash();
        let tx_hash = merkle_root_transactions(vec![coinbase.clone()]);
        let extra = Vec::from("Coinse base");

        let header = Header::new_mock(pre_hash, self.minter, tx_hash, pre_header.height + 1, next_time, Some(extra));
        Block::new(header, vec![coinbase])
    }

    fn coinbase_transaction(&self) -> Transaction {
        let nonce: u64 = random();
        let to = self.minter;
        let amount = random::<u64>();
        let gas_limit = random::<u64>();
        let gas_price = 1_u64;
        let payload = Vec::from(chrono::Local::now().to_string());

        let mut transaction = Transaction::new(nonce, to, amount, gas_limit, gas_price, payload);
        transaction.sign(self.chain.config.chain_id, &self.key_pair.secret());
        transaction
    }

    fn next_block(&self) -> (u64, Header) {
        let pre_block = self.chain.get_last_block();
        let pre_header = pre_block.header();
        let pre_timestamp = pre_header.time;
        let next_timestamp = pre_timestamp + self.chain.config.block_period.as_secs();
        let now_timestamp = chrono::Local::now().timestamp() as u64;
        trace!("now timestamp: {}, pre_timestamp: {}, next_timestamp: {}", now_timestamp, pre_timestamp, next_timestamp);
        if now_timestamp > next_timestamp {
            return (now_timestamp, pre_header.clone());
        }
        (next_timestamp, pre_header.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cryptocurrency_kit::ethkey::{Random, Generator};

    #[test]
    fn t_basecoin() {
        let nonce: u64 = random();
        let to = Address::from(199);
        let amount = random::<u64>();
        let gas_limit = random::<u64>();
        let gas_price = 1_u64;
        let payload = Vec::from(chrono::Local::now().to_string());

        let mut transaction = Transaction::new(nonce, to, amount, gas_limit, gas_price, payload);
        transaction.sign(100, Random.generate().unwrap().secret());

        let coinbase = transaction;
        let tx_hash = merkle_root_transactions(vec![coinbase.clone()]);
        println!("coin base hash: {:?}", tx_hash);
    }
}
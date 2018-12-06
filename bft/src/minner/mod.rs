use std::sync::Arc;

use actix::prelude::*;
use parking_lot::RwLock;
use crossbeam::{Sender, Receiver, channel::bounded};
use rand::random;
use cryptocurrency_kit::ethkey::{Address, KeyPair};
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::Hash;
use cryptocurrency_kit::crypto::CryptoHash;
use cryptocurrency_kit::crypto::hash;

use crate::{
    core::chain::Chain,
    core::tx_pool::TxPool,
    core::events::*,
    consensus::backend::Backend,
    consensus::consensus::Engine,
    types::Timestamp,
    types::block::{Block, Header},
    types::transaction::{Transaction, merkle_root_transactions},
};

pub struct Minner {
    minter: Address,
    key_pair: KeyPair,
    chain: Arc<Chain>,
    txpool: Box<TxPool>,
    engine: Box<Engine>,
    //    chain_event: Addr<ProcessSignals>,
    seal_tx: Sender<()>,
    seal_rx: Receiver<()>,
}

impl Actor for Minner {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Start minner actor");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("Minner actor has stoppped");
    }
}

impl Handler<ChainEvent> for Minner {
    type Result = ();
    fn handle(&mut self, msg: ChainEvent, ctx: &mut Self::Context) -> Self::Result {
        // stop current consensus
        self.seal_tx.send(()).unwrap();
        let seal = self.seal_rx.clone();
        self.mine(seal);
    }
}

impl Minner {
    fn mine(&mut self, abort: Receiver<()>) {
        let mut block = self.packet_next_block();
        self.engine.seal(&mut block, abort);
    }

    fn packet_next_block(&self) -> Block {
        let (next_time, pre_header) = self.next_block();
        let coinbase = self.coinbase_transaction();

        let pre_hash: Hash = pre_header.hash();
        let tx_hash = merkle_root_transactions(vec![coinbase.clone()]);

        let header = Header::new_mock(pre_hash, self.minter, tx_hash, pre_header.height + 1, next_time, None);
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
        if now_timestamp > next_timestamp {
            return (now_timestamp, pre_header.clone());
        }
        (next_timestamp, pre_header.clone())
    }
}
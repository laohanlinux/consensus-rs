use std::sync::Arc;

use parking_lot::RwLock;

use crate::{
    config::Config,
    error::{ChainError, ChainResult},
    types::block::Block,
};
use super::genesis::store_genesis_block;
use super::ledger::Ledger;

pub struct Chain {
    ledger: Arc<RwLock<Ledger>>,
    genesis: Option<Block>,
    config: Config,
}

impl Chain {
    pub fn new(config: Config, ledger: Arc<RwLock<Ledger>>) -> Self {
        Chain {
            ledger,
            config,
            genesis: None,
        }
    }

    pub fn insert_block() {}

    pub fn get_ledger(&self) -> &Arc<RwLock<Ledger>> {
        &self.ledger
    }

    pub fn get_genesis(&self) -> &Block {
        self.genesis.as_ref().unwrap()
    }

    pub fn store_genesis_block(&mut self) -> ChainResult {
        let result = store_genesis_block(self.config.genesis.as_ref().unwrap(), self.ledger.clone())
            .map_err(|err| ChainError::Unknown(err));
        if result.is_ok() {
            let genesis = {
                let ledger = self.ledger.read();
                ledger.get_genesis_block().unwrap().clone()
            };
            self.genesis = Some(genesis);
        }

        result
    }
}
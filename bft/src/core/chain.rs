use std::sync::Arc;

use actix::Addr;
use parking_lot::RwLock;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::Hash;

use crate::{
    config::Config,
    error::{ChainError, ChainResult},
    types::{Height, Validators, ValidatorArray, Validator, block::Block},
};
use super::genesis::store_genesis_block;
use super::ledger::Ledger;

pub struct Chain {
    ledger: Arc<RwLock<Ledger>>,
    //    subscriber: Addr<ProcessSignals>,
    genesis: Option<Block>,
    pub config: Config,
}

impl Chain {
    pub fn new(config: Config, ledger: Arc<RwLock<Ledger>>) -> Self {
        Chain {
            ledger,
//            subscriber: subscriber,
            config,
            genesis: None,
        }
    }

    pub fn insert_block() {}

    pub fn get_ledger(&self) -> &Arc<RwLock<Ledger>> {
        &self.ledger
    }

    pub fn get_last_height(&self) -> Height {
        self.ledger.read().get_last_block_height().clone()
    }

    pub fn get_last_block(&self) -> Block {
        self.ledger.read().get_last_block().clone()
    }

    pub fn get_last_hash(&self) -> Hash {
        self.ledger.read().get_last_block_hash().clone()
    }

    pub fn add_validators(&self, height: Height, validators: Vec<Address>) -> ChainResult {
        let validators = validators.iter().map(|address| Validator::new(*address)).collect();
        self.ledger.write().add_validators(validators);
        Ok(())
    }

    // FIXME: Opz avoid to copy validator memory
    pub fn get_validators(&self, height: Height) -> Validators {
        let ledger = self.ledger.read();
        ledger.get_validators(height).clone()
    }

    pub fn get_genesis(&self) -> &Block {
        self.genesis.as_ref().unwrap()
    }

    pub fn store_genesis_block(&mut self) -> ChainResult {
        let result = store_genesis_block(self.config.genesis.as_ref().unwrap(), self.ledger.clone())
            .map_err(ChainError::Unknown);
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
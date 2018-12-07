use std::sync::Arc;

use actix::prelude::*;
use parking_lot::RwLock;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::Hash;
use futures::Future;

use crate::{
    config::Config,
    error::{ChainError, ChainResult},
    types::{Height, Validators, ValidatorArray, Validator, block::Block, block::Header},
    subscriber::events::{ChainEvent, ChainEventSubscriber},
};
use super::genesis::store_genesis_block;
use super::ledger::Ledger;

pub struct Chain {
    ledger: Arc<RwLock<Ledger>>,
    subscriber: Addr<ChainEventSubscriber>,
    genesis: Option<Block>,
    pub config: Config,
}

impl Chain {
    pub fn new(config: Config, subscriber: Addr<ChainEventSubscriber>, ledger: Arc<RwLock<Ledger>>) -> Self {
        Chain {
            ledger,
            subscriber: subscriber,
            config,
            genesis: None,
        }
    }

    pub fn insert_block(&self, block: &Block) -> ChainResult {
        {
            let mut ledger = self.ledger.write();
            if ledger.get_block_by_height(block.height()).is_some() {
                trace!("{:?} has exists", block.hash());
                return Err(ChainError::Exists(block.hash()));
            }
            ledger.add_block(block);
        }
        let req1 = self.subscriber.send(ChainEvent::NewBlock(block.clone()));
        let req2 = self.subscriber.send(ChainEvent::NewHeader(block.header().clone()));
        Arbiter::spawn(req1.and_then(|_res| {
            futures::future::ok(())
        }).map_err(|err| panic!(err)));
        Arbiter::spawn(req2.and_then(|_res| {
            futures::future::ok(())
        }).map_err(|err| panic!(err)));

        Ok(())
    }

    pub fn get_ledger(&self) -> &Arc<RwLock<Ledger>> {
        &self.ledger
    }

    pub fn get_last_height(&self) -> Height {
        self.ledger.read().get_last_block_height().clone()
    }

    pub fn get_last_block(&self) -> Block {
        self.ledger.read().get_last_block().clone()
    }

    pub fn get_block_hash_by_height(&self, height: Height) -> Option<Hash> {
        self.ledger.read().get_block_hash_by_height(height)
    }

    pub fn get_header_by_height(&self, height: Height) -> Option<Header> {
        self.ledger.read().get_header_by_height(height)
    }

    pub fn get_last_hash(&self) -> Hash {
        self.ledger.read().get_last_block_hash().clone()
    }

    pub fn add_validators(&self, _height: Height, validators: Vec<Address>) -> ChainResult {
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
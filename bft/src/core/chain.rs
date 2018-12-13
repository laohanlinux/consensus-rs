use std::sync::Arc;

use ::actix::prelude::*;
use parking_lot::RwLock;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::Hash;
use futures::Future;

use crate::{
    config::Config,
    error::{ChainError, ChainResult},
    types::{Height, Validators, ValidatorArray, Validator, transaction::Transaction, block::Block, block::Header},
    subscriber::events::{ChainEvent, ChainEventCT::ProcessSignals, ChainEventCT::SubscribeMessage},
};
use super::genesis::store_genesis_block;
use super::ledger::Ledger;

pub struct Chain {
    ledger: Arc<RwLock<Ledger>>,
    subscriber: Addr<ProcessSignals>,
    genesis: Option<Block>,
    lock: RwLock<()>,
    pub config: Config,
}

impl Chain {
    pub fn new(config: Config, ledger: Arc<RwLock<Ledger>>) -> Self {
        let subscriber = Actor::create(|ctx| {
            ctx.set_mailbox_capacity(1024);
            ProcessSignals::new()
        });
        Chain {
            ledger,
            subscriber: subscriber,
            lock: RwLock::new(()),
            config,
            genesis: None,
        }
    }

    pub fn insert_block(&self, block: &Block) -> ChainResult {
        self.lock.write();
//        info!("Ready insert a new block, hash: {}, height: {}", block.hash().short(), block.height());
        {
            let mut ledger = self.ledger.write();
            if let Some(old_block) = ledger.get_block_by_height(block.height()) {
                return Err(ChainError::Exists(block.hash()));
            }
            let last_height = ledger.get_last_block_height();
            if last_height + 1 < block.height() {
                self.post_event(ChainEvent::SyncBlock(last_height + 1));
                return Err(ChainError::Unknown("Not found ancestor".to_owned()));
            }

            ledger.add_block(block);
        }
        self.subscriber.do_send(ChainEvent::NewBlock(block.clone()));
        self.subscriber.do_send(ChainEvent::NewHeader(block.header().clone()));
        Ok(())
    }

    pub fn insert_block_mock(block: &Block, ledger: Arc<RwLock<Ledger>>) -> ChainResult {
        info!("Ready insert a new block, hash: {}, height: {}", block.hash().short(), block.height());
        {
            let mut ledger = ledger.write();
            if let Some(old_block) = ledger.get_block_by_height(block.height()) {
                info!("{:#?}", old_block);
                return Err(ChainError::Exists(block.hash()));
            }
            ledger.add_block(block);
        }
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

    pub fn get_block_by_hash(&self, block_hash: &Hash) -> Option<Block> {
        self.ledger.read().get_block(block_hash)
    }

    pub fn get_block_by_height(&self, height: Height) -> Option<Block> {
        if let Some(hash) = self.get_block_hash_by_height(height) {
            self.get_block_by_hash(&hash)
        } else {
            None
        }
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.ledger.read().get_transactions()
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
        let ledger = self.ledger.write();
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
                let mut ledger = self.ledger.write();
                ledger.get_genesis_block().unwrap().clone()
            };
            self.genesis = Some(genesis);
        }

        result
    }

    pub fn get_subscriber(&self) -> Addr<ProcessSignals> {
        self.subscriber.clone()
    }

    pub fn subscriber_event(&self, recipient: Recipient<ChainEvent>) {
        let message = SubscribeMessage::new_subScribe(recipient);
        let request_fut = self.subscriber.send(message);
        Arbiter::spawn(
            request_fut
                .and_then(|_result| {
                    info!("Subsribe chain event successfully");
                    futures::future::ok(())
                })
                .map_err(|err| unimplemented!("{}", err)),
        );
    }

    pub fn post_event(&self, event: ChainEvent) {
        self.subscriber.try_send(event);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::random_dir;
    use cryptocurrency_kit::ethkey::{Generator, Random};
    use kvdb_rocksdb::Database;
    use crate::store::schema::Schema;
    use crate::core::ledger::{Ledger, LastMeta};
    use lru_time_cache::LruCache;
    use std::sync::Arc;
    use parking_lot::RwLock;
    use cryptocurrency_kit::crypto::EMPTY_HASH;


    #[test]
    fn t_batch() {
        let secret = Random.generate().unwrap();

        let database = Database::open_default(&random_dir()).map_err(|err| err.to_string()).unwrap();
        let schema = Schema::new(Arc::new(database));
        let mut ledger = Ledger::new(
            LastMeta::new_zero(),
            LruCache::with_capacity(1 << 10),
            LruCache::with_capacity(1 << 10),
            vec![],
            schema,
        );

        let mut header = Header::new(EMPTY_HASH, Address::from(10), EMPTY_HASH, EMPTY_HASH, EMPTY_HASH,
                                     0, 0, 0, 10, 10,
                                     chrono::Local::now().timestamp() as u64, None, Some(vec![12, 1]));
        let block = Block::new(header, vec![]);

        ledger.add_genesis_block(&block);
        ledger.reload_meta();

        let ledger = Arc::new(RwLock::new(ledger));

        (1_u64..10).for_each(|height| {
            let mut header = Header::new(EMPTY_HASH, Address::from(10), EMPTY_HASH, EMPTY_HASH, EMPTY_HASH,
                                         0, 0, height, 10, 10,
                                         chrono::Local::now().timestamp() as u64, None, Some(vec![12, 1]));
            let block = Block::new(header, vec![]);

            Chain::insert_block_mock(&block, ledger.clone());
        });

        let ledger = ledger.read();
        (1_u64..10).for_each(|height| {
            let block = ledger.get_block_by_height(height).unwrap();
            let block1 = ledger.get_block(&block.hash()).unwrap();
            println!("{:?}", block);
            println!("|{:?}", block1);
        });

//        let schema = ledger.get_schema();
//        for block in schema.blocks().iter() {
//            println!("{:?}", block);
//        }

        println!("last_block {:?}", ledger.get_last_block());
    }
}
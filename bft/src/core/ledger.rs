use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use kvdb_rocksdb::{Database, DatabaseConfig, DatabaseIterator};
use lru_time_cache::LruCache;
use parking_lot::RwLock;
use std::collections::HashMap;

use crate::{
    store::schema::Schema,
    types::block::{Block, Header},
    types::transaction::Transaction,
    types::{Height, Validator, ValidatorArray},
};

pub struct LastMeta {
    height: Height,
    block_hash: Hash,
    header: Header,
    block: Block,
}

impl LastMeta {
    pub fn new_zero() -> Self {
        Self::new(
            0,
            Hash::zero(),
            Header::zero_header(),
            Block::new(Header::zero_header(), vec![]),
        )
    }

    pub fn new(height: Height, block_hash: Hash, header: Header, block: Block) -> Self {
        LastMeta {
            height,
            block_hash,
            header,
            block,
        }
    }
}

/// it is not thread safe
pub struct Ledger {
    meta: LastMeta,
    header_cache: RwLock<LruCache<Hash, Header>>,
    block_cache: RwLock<LruCache<Hash, Block>>,
    genesis: Option<Block>,
    validators: Vec<Validator>,
    schema: Schema,
}

impl Ledger {
    pub fn new(
        meta: LastMeta,
        header_cache: LruCache<Hash, Header>,
        block_cache: LruCache<Hash, Block>,
        validators: Vec<Validator>,
        schema: Schema,
    ) -> Self {
        Ledger {
            meta,
            header_cache: RwLock::new(header_cache),
            block_cache: RwLock::new(block_cache),
            genesis: None,
            validators,
            schema,
        }
    }

    pub fn get_transaction(&self, tx_hash: &Hash) -> Option<Transaction> {
        self.schema.transaction().get(tx_hash)
    }

    pub fn get_genesis_block(&mut self) -> Option<&Block> {
        if self.genesis.is_some() {
            return self.genesis.as_ref();
        }
        let genesis = self.get_block_by_height(0);
        if genesis.is_none() {
            return None;
        }
        self.genesis.replace(genesis.unwrap());
        self.genesis.as_ref()
    }

    pub fn get_last_block_height(&self) -> &Height {
        &self.meta.height
    }

    pub fn get_last_block_header(&self) -> &Header {
        &self.meta.header
    }

    pub fn get_last_block(&self) -> &Block {
        &self.meta.block
    }

    pub fn get_last_block_hash(&self) -> &Hash {
        &self.meta.block_hash
    }

    pub fn get_block_hash_by_height(&self, height: Height) -> Option<Hash> {
        self.schema.block_hashes_by_height().get(height)
    }

    pub fn get_block_header(&self, block_hash: &Hash) -> Option<Header> {
        let mut cache = self.header_cache.write();
        if let Some(header) = cache.get_mut(block_hash) {
            return Some(header.clone());
        }

        if let Some(header) = self.schema.headers().get(block_hash) {
            return Some(header);
        }
        None
    }

    pub fn get_block(&self, block_hash: &Hash) -> Option<Block> {
        let mut cache = self.block_cache.write();
        let block = cache.get(block_hash);
        match block {
            Some(block) => Some(block.clone()),
            None => {
                if let Some(header) = self.schema.headers().get(block_hash) {
                    let block = Block::new2(header, vec![]);
                    cache.insert(*block_hash, block.clone());
                    Some(block)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        let tx = self.schema.transaction();
        let mut transactions = vec![];
        for tx in tx.iter() {
            transactions.push(tx.1.clone());
        }
        transactions
    }

    //  FIXME store it into schema
    pub fn get_validators(&self, _height: Height) -> &Vec<Validator> { &self.validators }

    pub fn get_block_by_height(&self, height: Height) -> Option<Block> {
        if let Some(hash) = self.schema.block_hash_by_height(height) {
            if let Some(block) = self.block_cache.write().get(&hash) {
                return Some(block.clone());
            }

            if let Some(header) = self.schema.headers().get(&hash) {
                return Some(Block::new2(header, vec![]));
                // TODO add transations
            }
        }
        None
    }

    pub fn get_header_by_height(&self, height: Height) -> Option<Header> {
        if let Some(block_hash) = self.schema.block_hash_by_height(height) {
            if let Some(header) = self.header_cache.write().get(&block_hash) {
                return Some(header.clone());
            }
            if let Some(header) = self.schema.headers().get(&block_hash) {
                // cache it
                self.header_cache
                    .write()
                    .insert(block_hash, header.clone());
                return Some(header);
            }
        }
        None
    }

    pub fn add_genesis_block(&mut self, block: &Block) {
        self.add_block(block);
        self.genesis = Some(block.clone());
    }

    pub fn add_block(&mut self, block: &Block) {
        let header = block.header();
        let hash = header.block_hash();
        if self.meta.header.height >= header.height && block.height() != 0 {
            return;
        }

        // persists
        {
            debug!("Write header");
            let mut header_db = self.schema.headers();
            header_db.put(&hash, header.clone());
        }

        // transactions
        {
            let mut tx_db = self.schema.transaction();
            debug!("Write transaction");
            for transaction in block.transactions() {
                tx_db.put(&transaction.hash(), transaction.clone());
            }
        }

        // height
        {
            let mut height_db = self.schema.block_hashes_by_height();
            debug!("Write height, hash:{:?}, height:{:?}", hash.short(), block.height());
            height_db.push(hash.clone());
            assert_eq!(height_db.last().unwrap(), hash);
            assert_eq!(height_db.len(), block.height() + 1);
        }

        // cache it
        {
            self.header_cache
                .get_mut()
                .insert(hash, header.clone());
            self.block_cache
                .get_mut()
                .insert(hash, block.clone());
        }

        // update last meta
        self.update_meta(block);
        info!("🔨🔨🔨Insert new block, hash:{:?}, height:{}, utime:{}, proposer:{:?}", hash.short(), header.height, header.time, header.proposer);
    }

    pub fn add_validators(&mut self, validators: Vec<Validator>) {
        let val_array = ValidatorArray::from(validators.clone());
        let mut validators_entry = self.schema.validators();
        validators_entry.set(val_array);
        // cache it
        self.validators = validators;
    }

    pub fn reload_meta(&mut self) {
        let hashes = self.schema.block_hashes_by_height();
        let last_hash = hashes.last().unwrap();
        let last_block = self.get_block(&last_hash).unwrap();
        self.update_meta(&last_block);
    }

    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }

    fn update_meta(&mut self, block: &Block) {
        let header = block.header();
        self.meta.header = header.clone();
        self.meta.height = header.height;
        self.meta.block_hash = block.hash();
        self.meta.block = block.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::{self, Write};

    #[test]
    fn db() {
        let dir = env::temp_dir();
        let db = Database::open_default(dir.to_str().unwrap()).unwrap();

        let mut tx = db.transaction();
        (0..100).for_each(|idx| {
            let (key, value) = (format!("tx_{:?}", idx), format!("{:?}", idx + 1));
            tx.put(None, key.as_bytes(), value.as_bytes());
        });
        db.write(tx).unwrap();
        db.flush().unwrap();
        db.iter_from_prefix(None, b"tx_").unwrap().for_each(|kv| {
            println!(
                "{:?}, {:?}",
                String::from_utf8_lossy(&kv.0),
                String::from_utf8_lossy(&kv.1)
            );
        });
    }

    #[test]
    fn tt() {
        use cryptocurrency_kit::storage::values::StorageValue;
        let str = r#"{
                    "prev_hash":[239,84,65,205,122,68,74,217,21,194,14,134,190,146,248,124,220,27,127,128,145,89,69,4,92,163,71,243,216,123,69,126],
                    "proposer":"0x72d5c75fd6703414aa87f79b3e4797dd09cd9251","root":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                    "tx_hash":[85,135,229,162,129,149,77,84,152,42,89,35,51,202,149,213,162,87,24,41,123,46,8,200,101,189,235,79,197,110,19,131],
                    "receipt_hash":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                    "bloom":0,"difficulty":0,"height":1,"gas_limit":0,"gas_used":0,"time":1544610951,"extra":null,
                    "votes":[[39,6,5,27,54,114,178,192,185,183,101,78,241,85,203,15,132,252,31,126,182,20,151,174,124,227,133,105,159,235,61,112,71,19,198,253,44,112,239,142,141,101,247,157,16,138,136,74,219,113,137,206,69,207,245,13,186,9,223,34,238,16,21,122,1],[232,108,59,252,67,6,209,191,45,176,232,22,248,211,56,45,117,155,177,141,190,238,162,186,58,201,141,251,70,237,72,23,102,171,167,28,52,110,33,131,142,51,201,244,187,33,178,183,9,202,68,190,194,84,122,235,101,243,31,48,254,161,38,128,1],[210,214,152,8,11,235,229,51,147,57,148,247,105,24,28,234,123,202,142,112,18,19,51,61,127,102,39,215,146,2,61,184,116,245,59,64,74,191,79,143,153,239,88,199,108,216,176,158,200,163,14,44,104,212,104,192,53,98,134,133,4,81,57,96,1]]}"#;
        let header: Header = serde_json::from_str(str).unwrap();

        println!("{:?}", header.into_bytes());
    }

    #[test]
    fn ledger() {
        use cryptocurrency_kit::storage::values::StorageValue;
        use std::borrow::Cow;
        let dir = crate::common::random_dir();
        let db = Database::open_default("/tmp/block/c1").unwrap();
        let value = db.get(None, &vec![99, 111, 114, 101, 46, 104, 101, 97, 100, 101, 114, 115, 94, 56, 218, 57, 251, 141, 68, 52, 105, 104, 106, 50, 235, 251, 198, 31, 101, 90, 125, 224, 155, 158, 52, 87, 156, 67, 214, 189, 102, 207, 161, 54]).unwrap();
        let header = Header::from_bytes(Cow::from(value.unwrap().as_ref()));
        println!("---> {:?}", header);
//        let mut ledger = Ledger::new();
    }
}

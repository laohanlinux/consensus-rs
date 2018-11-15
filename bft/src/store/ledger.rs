use std::sync::RwLock;

use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use kvdb_rocksdb::{Database, DatabaseConfig, DatabaseIterator};
use lru_time_cache::LruCache;

use types::block::{Block, Header};
use types::transaction::Transaction;
use types::{Height, Validator};

use super::schema::Schema;

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
            validators,
            schema,
        }
    }

    pub fn get_transaction(&self, tx_hash: &Hash) -> Option<Transaction> {
        self.schema.transaction().get(tx_hash)
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
        let mut cache = self.header_cache.write().unwrap();
        if let Some(header) = cache.get_mut(block_hash) {
            return Some(header.clone());
        }

        if let Some(block) = self.schema.blocks().get(block_hash) {
            return Some(block.header().clone());
        }
        None
    }

    pub fn get_block(&self, block_hash: &Hash) -> Option<Block> {
        let mut cache = self.block_cache.write().unwrap();
        let block = cache.get(block_hash);
        if block.is_none() {
            let db = self.schema.blocks();
            let block = db.get(block_hash);
            if block.is_some() {
                cache.insert(*block_hash, block.as_ref().unwrap().clone());
                return block;
            }
        }
        None
    }

    //  FIXME store it into schema
    pub fn get_validators(&self, height: Height) -> &Vec<Validator> {
        &self.validators
    }

    pub fn get_block_by_height(&self, height: Height) -> Option<Block> {
        if let Some(hash) = self.schema.block_hash_by_height(height) {
            if let Some(block) = self.block_cache.write().unwrap().get(&hash) {
                return Some(block.clone());
            }
            if let Some(block) = self.schema.blocks().get(&hash) {
                // cache it
                self.block_cache
                    .write()
                    .unwrap()
                    .insert(hash, block.clone());
                return Some(block.clone());
            }
        }
        None
    }

    pub fn get_header_by_height(&self, height: Height) -> Option<Header> {
        if let Some(block_hash) = self.schema.block_hash_by_height(height) {
            if let Some(header) = self.header_cache.write().unwrap().get(&block_hash) {
                return Some(header.clone());
            }
            if let Some(block) = self.schema.blocks().get(&block_hash) {
                // cache it
                self.header_cache
                    .write()
                    .unwrap()
                    .insert(block_hash, block.header().clone());
                return Some(block.header().clone());
            }
        }
        None
    }

    pub fn add_block(&mut self, block: &Block) {
        let header = block.header();
        let hash = header.hash();
        if self.meta.header.height >= header.height {
            return;
        }

        // update last meta
        self.meta.header = header.clone();
        self.meta.height = header.height;
        self.meta.block_hash = hash;
        self.meta.block = block.clone();

        // persists
        let mut block_db = self.schema.blocks();
        block_db.put(&hash, block.clone());
        let mut heigh_db = self.schema.block_hashes_by_height();
        heigh_db.push(hash.clone());
        // cache it
        self.header_cache
            .get_mut()
            .unwrap()
            .insert(hash, header.clone());
        self.block_cache
            .get_mut()
            .unwrap()
            .insert(hash, block.clone());
    }

    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    #[test]
    fn db() {
        use std::env;

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
}

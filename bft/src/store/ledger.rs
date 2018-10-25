use cryptocurrency_kit::crypto::{hash, Hash, CryptoHash};
use kvdb_rocksdb::{Database, DatabaseConfig, DatabaseIterator};
use lru_time_cache::LruCache;

use types::block::{Block, Header};
use types::transaction::Transaction;
use types::{Height, Validator};

pub struct LastMeta {
    height: Height,
    block_hash: Hash,
    header: Header,
    block: Block,
}

impl LastMeta {
    pub fn new_zero() -> Self {
        Self::new(0, Hash::zero(), Header::zero_header(),
                  Block::new(Header::zero_header(), vec![], None))
    }

    pub fn new(height: Height, block_hash: Hash, header: Header, block: Block) -> Self {
        LastMeta{
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
    header_cache: LruCache<Hash, Header>,
    block_cache: LruCache<Hash, Block>,
    db: Database,
}

impl Ledger {
    pub fn new(meta: LastMeta, header_cache: LruCache<Hash, Header>, block_cache: LruCache<Hash, Block>, db: Database) -> Self {
        Ledger {
            meta,
            header_cache,
            block_cache,
            db,
        }
    }

    pub fn get_transaction(tx_hash: &Hash) -> Option<Transaction> {
        None
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

    pub fn get_block_header(block_hash: &Hash) -> Option<Header> {
        None
    }

    pub fn get_block(&self, block_hash: &Hash) -> Option<Block> {
//        self.block_cache.get(block_hash).unwrap_or_else( ||{
//            self.db.get()
//        })
        None
    }

    pub fn get_validators(height: &Height) -> Vec<Validator> {
        vec![]
    }

    pub fn get_block_by_height(height: &Height) -> Option<Block> {
        None
    }

    pub fn get_header_by_height(height: &Height) -> Option<Header> {
        None
    }

    pub fn add_block(&mut self, block: &Block) {
        let header = block.header();
        let hash = header.hash();
        if self.meta.header.height >= header.height {
            return
        }

        // update last meta
        self.meta.header = header.clone();
        self.meta.height = header.height;
        self.meta.block_hash = hash.clone();
        self.meta.block = block.clone();

        // cache it
        self.header_cache.insert(hash.clone(), header.clone());
        self.block_cache.insert(hash.clone(), block.clone());
    }

    pub fn get_db(&self) -> &Database {
        &self.db
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

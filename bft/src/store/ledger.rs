use kvdb_rocksdb::{Database, DatabaseConfig, DatabaseIterator};
use cryptocurrency_kit::crypto::{Hash, hash};

use types::{Height, Validator};
use types::transaction::Transaction;
use types::block::{Block, Header};

pub struct Ledger {}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger{}
    }

    pub fn get_transaction(tx_hash: &Hash) -> Option<Transaction>{
        None
    }

    pub fn get_block_header(block_hash: &Hash) -> Option<Header> {
        None
    }

    pub fn get_block(block_hash: &Hash) -> Option<Block> {
        None
    }

    pub fn get_validators(height: &Height) -> Vec<Validator> {
        vec![]
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
            println!("{:?}, {:?}", String::from_utf8_lossy(&kv.0), String::from_utf8_lossy(&kv.1));
        });
    }
}

use std::sync::Arc;

use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::ethkey::Address;
use kvdb_rocksdb::Database;

use super::entry::Entry;
use super::list_index::ListIndex;
use super::map_index::MapIndex;
use crate::{
    types::block::{Block, Header},
    types::{Validator, ValidatorArray, HashesEntry, Bloom, Height, transaction::Transaction},
};

macro_rules! define_name {
    (
        $(
            $name:ident => $value:expr;
        )+
    ) => (
        $(const $name: &str = concat!("core.", $value);)*
    );
}

define_name!(
    TRANSACTIONS => "transaction_hash";
    TRANSACTIONS_HASH => "transaction_block";
    BLOCKS => "blocks";
    HEADERS => "headers";
    BLOCK_HASHES_BY_HEIGHT => "block_hashes_by_height";
    BLOCK_TRANSACTIONS => "block_transactions";
    PRECOMMITS => "precommits";
    CONFIGS => "configs";
    CONSENSUS_MESSAGE_CACHE => "consensus_message_cache";
    CONSENSUS_ROUND => "consensus_round";
    VALIDATORS => "validators";
);

struct TxLocation {
    block_height: Height,
    position_in_block: u64,
}

pub struct Schema {
    db: Arc<Database>,
}

impl Schema {
    pub fn new(db: Arc<Database>) -> Self {
        Schema { db }
    }

    pub fn transaction(&self) -> MapIndex<Hash, Transaction> {
        MapIndex::new(TRANSACTIONS, self.db.clone())
    }

    pub fn transaction_hashes(&self) -> MapIndex<Hash, HashesEntry> {
        MapIndex::new(TRANSACTIONS_HASH, self.db.clone())
    }

    pub fn blocks(&self) -> MapIndex<Hash, Block> {
        MapIndex::new(BLOCKS, self.db.clone())
    }

    pub fn headers(&self) -> MapIndex<Hash, Header> {
        MapIndex::new(HEADERS, self.db.clone())
    }

    pub fn block_hashes_by_height(&self) -> ListIndex<Hash> {
        ListIndex::new(BLOCK_HASHES_BY_HEIGHT, self.db.clone())
    }

    pub fn block_hash_by_height(&self, height: Height) -> Option<Hash> {
        self.block_hashes_by_height().get(height)
    }

    pub fn last_block(&self) -> Block {
        let hash = self.block_hashes_by_height()
            .last()
            .expect("An attempt to get the `last_block` during creating the genesis block .");
        let header = self.headers().get(&hash).unwrap();
        Block::new2(header, vec![])
    }

    pub fn validators(&self) -> Entry<ValidatorArray> {
        Entry::new(VALIDATORS, self.db.clone())
    }

    /// Returns the height of the last committed block.
    ///
    /// #Panic
    ///
    /// Panic if the "genesis block" was not created
    ///
    /// (len - 1) because including a genesis hash
    pub fn height(&self) -> Height {
        let len = self.block_hashes_by_height().len();
        assert!(
            len > 0,
            "An attempt to get the actual `height` during creating the genesis block."
        );
        len - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::random_dir;
    use cryptocurrency_kit::ethkey::{Generator, KeyPair, Random, Secret};
    use std::io::{self, Write};
    use std::sync::Arc;

    fn random_secret() -> Secret {
        let key_pair = Random.generate().unwrap();
        key_pair.secret().clone()
    }

    #[test]
    fn tschema() {
        let db = Arc::new(Database::open_default(&random_dir()).unwrap());
        let schema = Schema::new(db.clone());

        /// block_hashes_by_height
        {
            let mut ledger = schema.block_hashes_by_height();
            (0..100).for_each(|idx| {
                ledger.push(idx.hash());
            });
            let iter = ledger.iter();
            iter.for_each(|hash| {
                writeln!(io::stdout(), "{}", hash.short()).unwrap();
            });
            writeln!(io::stdout(), "-->{:?}", ledger.get(1).unwrap()).unwrap();
        }

        // transaction
        {
            let mut tx = schema.transaction();
            let mut zero_tx = Transaction::new(
                1024,
                ::cryptocurrency_kit::ethkey::Address::from(100),
                0,
                0,
                0,
                vec![1, 2, 3],
            );
            zero_tx.sign(1, &random_secret());
            writeln!(
                io::stdout(),
                "transaction hash ===> {:#?}",
                zero_tx.hash().short()
            )
                .unwrap();
            {
                let buf = zero_tx.clone().into_bytes();
                let _zero_tx1 = Transaction::from_bytes(::std::borrow::Cow::from(buf));
            }
            tx.put(&zero_tx.hash(), zero_tx.clone());
            let zero_tx1 = schema.transaction().get(&zero_tx.hash());
            assert_eq!(zero_tx1.is_some(), true);
            writeln!(io::stdout(), "{:#?}", zero_tx1.unwrap()).unwrap();
        }
    }
}

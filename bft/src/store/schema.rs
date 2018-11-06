use std::sync::Arc;

use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use kvdb_rocksdb::Database;

use super::map_index::MapIndex;
use types::block::{Block, Header};
use types::transaction::Transaction;
use types::{Bloom, Height};

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
    TRANSACTIONS => "transaction";
    BLOCKS => "blocks";
    BLOCK_HASHES_BY_HEIGHT => "block_hashes_by_height";
    BLOCK_TRANSACTIONS => "block_transactions";
    PRECOMMITS => "precommits";
    CONFIGS => "configs";
    CONSENSUS_MESSAGE_CACHE => "consensus_message_cache";
    CONSENSUS_ROUND => "consensus_round";
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

    pub fn blocks(&self) -> MapIndex<Hash, Block> {
        MapIndex::new(BLOCKS, self.db.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::random_dir;
    use cryptocurrency_kit::ethkey::{KeyPair, Secret, Random, Generator};
    use std::io::{self, Write};
    use std::sync::Arc;

    fn random_secret() -> Secret {
        let key_pair = Random.generate().unwrap();
        key_pair.secret().clone()
    }

    #[test]
    fn tschema() {
        writeln!(io::stdout(), "-----------------------").unwrap();
        let db = Arc::new(Database::open_default(&random_dir()).unwrap());
        let mut schema = Schema::new(db.clone());

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
            tx.put(&zero_tx.hash(), zero_tx.clone());
            let zero_tx1 = schema.transaction().get(&zero_tx.hash());
//            assert_eq!(zero_tx1.is_some(), true);
//            assert_eq!(zero_tx1.as_ref().unwrap(), &zero_tx);
            writeln!(io::stdout(), "===> {:#?}", zero_tx1).unwrap();
        }
    }
}

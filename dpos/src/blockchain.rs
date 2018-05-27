use std::{fmt, iter, mem, panic};
use std::sync::Arc;
use std::collections::{BTreeMap, HashMap};
use std::error::Error as StdError;
use std::io::Cursor;
use std::borrow::Cow;

use chain::storage::{Database, Snapshot, Patch, Fork};
use chain::blockchain::Schema;

/// Exonum blockchain instance with the concrete services set and data storage.
/// Only blockchains with the identical set of services and genesis block can be combined
/// into the single network.
pub struct Blockchain {
    db: Arc<Database>,
}

impl Blockchain {
    /// Constructs a blockchain for the given `storage` and list of `services`.
    pub fn new<D: Into<Arc<Database>>>(storage: D) -> Blockchain {
        Blockchain {
            db: storage.into(),
        }
    }

    /// Creates a readonly snapshot of the current storage state.
    pub fn snapshot(&self) -> Box<Snapshot> { self.db.snapshot() }

    /// Creates snapshot of the current state that can be later committed
    /// via `merge` method.
    pub fn fork(&self) -> Fork { self.db.fork() }

    /// Commits changes from the patch to the blockchain storage.
    /// See [`Fork`](../storage/struct.Fork.html) for details.
    pub fn merge(&mut self, patch: Patch) -> Result<(), Error> { self.db.merge(patch) }
}

impl fmt::Debug for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Blockchain(..")
    }
}

impl Clone for Blockchain {
    fn clone(&self) -> Blockchain {
        Blockchain {
            db: Arc::clone(&self.db),
        }
    }
}

impl Blockchain {
    /// Creates snapshot of the current storage state that can be later committed into storage
    /// via `merge` method.
    pub fn fork(&self) -> Fork {
        self.db.fork()
    }

    /// Commits changes from the patch to the blockchain storage.
    /// See [`Fork`](../storage/struct.Fork.html) for details.
    pub fn merge(&mut self, patch: Patch) -> Result<(), Error> {
        self.db.merge(patch)
    }

    /// Returns the hash of latest committed block.
    ///
    /// # Panics
    ///
    /// - If the genesis block was not committed.
    pub fn last_hash(&self) -> Hash {
        Schema::new(&self.snapshot())
            .block_hashes_by_height()
            .last()
            .unwrap_or_else(Hash::default)
    }

    /// Returns the latest committed block.
    ///
    /// # Panics
    ///
    /// - If the genesis block was not committed.
    pub fn last_block(&self) -> Block {
        Schema::new(&self.snapshot()).last_block()
    }


    /// Creates and commits the genesis block for the given genesis configuration.
    fn create_genesis_block(&mut self, cfg: ConsensusConfig) -> Result<(), Error> {
        let patch = {
            let mut fork = self.fork();
            {
                let mut schema = Schema::new(&mut fork);
                if schema.block_hash_by_height(Height::zero()).is_some() {
                    return Ok(());
                }
            }
            self.merge(fork.into_patch())?;

            self.create_patch(vec![],Height::zero(), 0, &[]).1
        };

        self.merge(patch)?;
        Ok(())
    }

    /// Executes the given transactions from pool.
    /// Then it collects the resulting changes from the current storage state and returns them
    /// with the hash of resulting block.
    pub fn create_patch(&self, generator_id: Vec<u8>, height: Height, timestamp: i64, tx_hashes: &[Hash]) -> (Hash, Patch) {
        // Create fork
        let mut fork: Fork = self.fork();

        let block_hash = {
            // Get last hash.
            let last_hash = self.last_hash();
            // Save & execute transactions.
            for (index, hash) in tx_hashes.iter().enumerate() {
                self.execute_transaction(*hash, height, index, &mut fork)
                    // Execution could fail if the transaction
                    // cannot be deserialized or it isn't in the pool.
                    .expect("Transaction not found in the database");
            }

            // Get tx & state hash.
            let (tx_hash, state_hash) = {
                let schema = Schema::new(&fork);
                let state_hashes = {
                    let vec_core_state = schema.core_state_hash();
                    let mut state_hashes = Vec::new();
                };
                let tx_hash = schema.block_transactions(height).merkle_root();

                (tx_hash, state_hash)
            };

            // Create block.
            let block = Block::new(
                SCHEMA_MAJOR_VERSION,
                0, //proposer_id,
                generator_id,
                height,
                timestamp,
                tx_hashes.len() as u32,
                &last_hash,
                &tx_hash,
                &state_hash,
            );

            trace!("execute block = {:?}", block);

            // Calcute block hash.
            let block_hash = block.hash();
            // update height.
            let mut schema = Schema::new(&mut fork);
            schema.block_hash_by_height_mut().push(block_hash);
            // Save block.
            schema.blocks_mut().put(&block_hash, block);

            block_hash
        };

        (block_hash, fork.into_patch())
    }

    // TODO: Opz
    fn execute_transaction(
        &self,
        tx_hash: Hash,
        height: Height,
        index: usize,
        fork: &mut Fork,
    ) -> Result<(), failure::Error> {
        Ok(())
    }
}

// TODO: use macro reimplements
impl StorageKey for DposBlock {
    fn size(&self) -> usize {
        self.get_size()
    }

    fn write(&self, buffer: &mut [u8]) {
        let mut writer = Writer::new(Cursor::new(buffer));
        self.write_message(&mut writer).unwrap();
    }

    fn read(buffer: &[u8]) -> Self::Owned {
        let mut reader = Reader::from_bytes(buffer.to_vec());
        reader.read(DposBlock::from_reader).unwrap()
    }
}

impl StorageValue for DposBlock {
    fn into_bytes(self) -> Vec<u8> {
        let capacity = self.get_size();
        let mut buffer = Vec::with_capacity(capacity);
        buffer.extend(iter::repeat(0).take(capacity));
        {
            let mut writer = Writer::new(&mut buffer);
            self.write_message(&mut writer).unwrap();
        }
        buffer
    }

    fn from_bytes(value: Cow<[u8]>) -> Self {
        let mut reader = Reader::from_bytes(value.to_vec());
        reader.read(DposBlock::from_reader).unwrap()
    }
}

impl CryptoHash for DposBlock {
    fn hash(&self) -> Hash {
        let block_size = self.get_size();
        let mut buffer = Vec::with_capacity(block_size);
        buffer.extend(iter::repeat(0).take(block_size));
        self.write(&mut buffer);
        crypto::hash(&buffer)
    }
}


#[cfg(test)]
mod tests {
    use bytes::BufMut;
    use std::io::Cursor;
    use prost::Message;
    use storage::{StorageKey, StorageValue};
    use quick_protobuf::{Writer, Reader, MessageRead, MessageWrite};

    use std::collections::HashMap;
    use serde::{Deserialize, Serialize};
    use rmp_serde::{Deserializer, Serializer};

    use std::io::{self, Write};
    use std::iter;

    use super::DposBlock as Block;

    #[test]
    fn test_storage_key_for_message_pack() {
        #[derive(Debug, PartialEq, Deserialize, Serialize)]
        struct Human {
            age: u32,
            name: String,
        }

        let mut buf = vec![];
        assert_eq!(buf.len(), 0);
        let val = Human {
            age: 42,
            name: "John".into(),
        };

        val.serialize(&mut Serializer::new(&mut buf)).unwrap();
        assert!(buf.len() > 1);
    }

    #[test]
    fn test_storage_key_for_block() {
        let mut block = Block::default();
        block.height = 1_000;
        block.timestamp = 2_000;

        let block_size = block.get_size() - 2;
        let mut buffer = Vec::with_capacity(block_size);
        buffer.extend(iter::repeat(0).take(block_size));
        block.write(&mut buffer);

        writeln!(io::stdout(), "{}", buffer[0]).unwrap();

        let new_block: Block = Block::read(&buffer);
        assert_eq!(new_block.height, block.height);
    }
}

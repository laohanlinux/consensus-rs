// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The module containing building blocks for creating blockchains powered by
//! the Exonum framework.
//!
//! Services are the main extension point for the Exonum framework. To create
//! your service on top of Exonum blockchain you need to do the following:
//!
//! - Define your own information schema.
//! - Create one or more transaction types using the [`transactions!`] macro and
//!   implement the [`Transaction`] trait for them.
//! - Create a data structure implementing the [`Service`] trait.
//! - Write API handlers for the service, if required.
//!
//! You may consult [the service creation tutorial][doc:create-service] for a more detailed
//! manual on how to create services.
//!
//! [`transactions!`]: ../macro.transactions.html
//! [`Transaction`]: ./trait.Transaction.html
//! [`Service`]: ./trait.Service.html
//! [doc:create-service]: https://exonum.com/doc/get-started/create-service
pub use self::block::{Block, SCHEMA_MAJOR_VERSION, BlockProof};
pub use self::schema::Schema;
pub use self::transaction::{Transaction, TransactionResult};
pub use self::genesis::GenesisConfig;
pub use self::config::ConsensusConfig;

use vec_map::VecMap;
use byteorder::{ByteOrder, LittleEndian};
use mount::Mount;
use failure;

use std::{fmt, iter, mem, panic};
use std::sync::Arc;
use std::collections::{BTreeMap, HashMap};
use std::error::Error as StdError;
use std::io::Cursor;
use std::borrow::Cow;

use messages::RawMessage;
use crypto::{self, CryptoHash, Hash};
use storage::{Database, Error, Fork, Patch, Snapshot, StorageKey, StorageValue};
use helpers::{Height, Round, ValidatorId};
use encoding::Error as MessageError;

mod block;
#[macro_use]
mod transaction;
#[macro_use]
mod schema;
mod genesis;
mod config;


/// Exonum blockchain instance with the concrete services set and data storage.
/// Only blockchains with the identical set of services and genesis block can be combined
/// into the single network.
pub struct Blockchain {
    db: Arc<Database>,
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
                    let mut state_hashes:Vec<(Hash, Hash)> = Vec::new();
                    state_hashes
                };
                let state_hash = {
                    let mut sum_table = schema.state_hash_aggregator_mut();
                    for (key, hash) in state_hashes {
                        sum_table.put(&key, hash)
                    }
                    sum_table.merkle_root()
                };

                let tx_hash = schema.block_transactions(height).merkle_root();

                (tx_hash, state_hash)
            };

            // Create block.
            let block = Block::new(
                SCHEMA_MAJOR_VERSION,
                ValidatorId(0), //proposer_id,
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
            schema.block_hashes_by_height_mut().push(block_hash);
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
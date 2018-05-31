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
use failure;
use time::{self, Timespec, Duration};
use chrono::*;

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
use consensue::{slot, delegates};

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
    fn create_genesis_block(&mut self, cfg: GenesisConfig) -> Result<(), Error> {
        let patch = {
            let mut fork = self.fork();
            {
                let mut schema = Schema::new(&mut fork);
                if schema.block_hash_by_height(Height::zero()).is_some() {
                    return Ok(());
                }
            }
            self.merge(fork.into_patch())?;

            // time --> slot ---> delegates
            let timestamp = Timespec::new(cfg.genesis_timestamp, 0);
            let epoch_time = slot::get_time(timestamp);
            let block_slot = slot::get_slot_number(epoch_time);
            let (delegate_id, _) = delegates::get_block_slot_data(block_slot, Height::zero()).unwrap();
            let delegate_id = delegate_id.to_string();
            self.create_patch(delegate_id.into_bytes(), Height::zero(), cfg.genesis_timestamp, &[]).1
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
                let mut schema = Schema::new(&mut fork);

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
    use storage::{Database, MemoryDB, StorageKey, StorageValue};
    use super::slot;

    use std::collections::HashMap;
    use serde::{Deserialize, Serialize};

    use std::io::{self, Write};
    use std::iter;

    #[test]
    fn test_genesis(){
        let database = MemoryDB::new();
        let mut bc = super::Blockchain::new(database);
        bc.create_genesis_block(super::GenesisConfig::new());
        writeln!(io::stdout(), "{}", bc.last_hash()).unwrap();
        let block = bc.last_block();
        writeln!(io::stdout(), "{:#?}", block);
    }

    #[test]
    fn test_generate_block(){
        let database = MemoryDB::new();
        let genesis_config = super::GenesisConfig::new();
        let mut bc = super::Blockchain::new(database);

        {
            bc.create_genesis_block(super::GenesisConfig::new());
        }

        let epoch = slot::get_time(super::Timespec::new(genesis_config.genesis_timestamp,0));
        let mut current_slot = slot::get_slot_number(epoch);
        let mut current_height = 0;
        for height in range (1, 100) {
            current_height = current_height + height;
            current_slot = current_height + 1;
            let sleep_time = slot::get_slot_time(current_slot);


            // time --> slot ---> delegates
            let next_slot = super::delegates::get_block_slot_data();
            let epoch_time = slot::get_time(timestamp);
            let block_slot = slot::get_slot_number(epoch_time);
            let (delegate_id, _) = delegates::get_block_slot_data(block_slot, Height::zero()).unwrap();
            let delegate_id = delegate_id.to_string();
            self.create_patch(delegate_id.into_bytes(), Height::zero(), cfg.genesis_timestamp, &[]).1
        }
    }
}
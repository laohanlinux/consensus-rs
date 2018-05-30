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

use crypto::{CryptoHash, Hash, PublicKey};
use messages::{RawMessage};
use storage::{Entry, Fork, KeySetIndex, ListIndex, MapIndex, MapProof, ProofListIndex,
              ProofMapIndex, Snapshot};
use helpers::{Height, Round};
use super::{Block, Blockchain, TransactionResult};

/// Defines `&str` constants with given name and value.
macro_rules! define_names {
    (
        $(
            $name:ident => $value:expr;
        )+
    ) => (
        $(const $name: &str = concat!("core.", $value);)*
    );
}

define_names!(
    TRANSACTIONS => "transactions";
    TRANSACTION_RESULTS => "transaction_results";
    TRANSACTIONS_POOL => "transactions_pool";
    TRANSACTIONS_LOCATIONS => "transactions_locations";
    BLOCKS => "blocks";
    BLOCK_HASHES_BY_HEIGHT => "block_hashes_by_height";
    BLOCK_TRANSACTIONS => "block_transactions";
    PRECOMMITS => "precommits";
    CONFIGS => "configs";
    CONFIGS_ACTUAL_FROM => "configs_actual_from";
    STATE_HASH_AGGREGATOR => "state_hash_aggregator";
    PEERS_CACHE => "peers_cache";
    CONSENSUS_MESSAGES_CACHE => "consensus_messages_cache";
    CONSENSUS_ROUND => "consensus_round";
);

encoding_struct!(
    /// Configuration index.
    struct ConfigReference{
        /// The height, starting from which this configuration becomes actual.
        actual_from: Height,
        /// Hash of the configuration contents that serialized as raw bytes vec.
        cfg_hash: &Hash,
    }
);

encoding_struct!(
    /// Transaction location in block.
    struct TxLocation{
        /// Height of block in the blockchain.
        block_height: Height,
        /// Index in block.
        position_in_block: u64,
    }
);


/// Information schema for `exonum-core`.
#[derive(Debug)]
pub struct Schema<T>{
    view: T,
}

impl <T> Schema<T>
where T: AsRef<Snapshot>,
{
    /// Constructs information schema for the given `snapshot`.
    pub fn new(snapshot: T) -> Schema<T> { Schema { view: snapshot } }

    /// Returns table represents a map from transaction hash into raw transaction message.
    pub fn transactions(&self) -> MapIndex<&T, Hash, RawMessage> {
        MapIndex::new(TRANSACTIONS, &self.view)
    }

    /// Returns table that represents a map from transaction hash into execution result.
    pub fn transaction_results(&self) -> ProofMapIndex<&T, Hash, TransactionResult> {
        ProofMapIndex::new(TRANSACTION_RESULTS, &self.view)
    }

    /// Returns table that represents a set of uncommitted transactions hashes.
    pub fn transactions_pool(&self) -> KeySetIndex<&T, Hash> {
        KeySetIndex::new(TRANSACTIONS_POOL, &self.view)
    }


    /// Returns number of transactions in the pool
    #[cfg_attr(feature = "cargo-clippy", allow(let_and_return))]
    pub fn transactions_pool_len(&self) -> usize {
        let pool: KeySetIndex<&T, Hash> = self.transactions_pool();
        // TODO: change count to other method with o(1) complexity. (ECR-977)
        let count: usize = pool.iter().count();
        count
    }

    /// Returns table that keeps the block height and tx position inside block for every
    /// transaction hash.
    pub fn transactions_locations(&self) -> MapIndex<&T, Hash, TxLocation> {
        MapIndex::new(TRANSACTIONS_LOCATIONS, &self.view)
    }

    /// Returns table that stores block object for very block height.
    pub fn blocks(&self) -> MapIndex<&T, Hash, Block> {
        MapIndex::new(BLOCKS, &self.view)
    }

    /// Returns table that keeps block hash for the corresponding height.
    pub fn block_hashes_by_height(&self) -> ListIndex<&T, Hash> {
        ListIndex::new(BLOCK_HASHES_BY_HEIGHT, &self.view)
    }

    /// Returns table that keeps a list of transactions for the each block.
    pub fn block_transactions(&self, height: Height) -> ProofListIndex<&T, Hash> {
        let height: u64 = height.into();
        ProofListIndex::new_in_family(BLOCK_TRANSACTIONS, &height, &self.view)
    }

    /// Returns block hash for the given height.
    pub fn block_hash_by_height(&self, height: Height) -> Option<Hash> {
        self.block_hashes_by_height().get(height.into())
    }

    /// Returns latest committed block.
    ///
    /// #Panics
    ///
    /// Panics if the "genesis block" was not created
    pub fn last_block(&self) -> Block {
        let hash: Hash = self.block_hashes_by_height()
            .last()
            .expect("An attempt to get the `last_block` during creating the genesis block.");
        self.blocks().get(&hash).unwrap()
    }


    /// Returns height of the latest committed block.
    ///
    /// # Panics
    ///
    /// Panics if the "genesis block" was not created.
    pub fn height(&self) -> Height {
        let len: u64 = self.block_hashes_by_height().len();
        assert!(len > 0, "An attempt to get the actual `height` during creating the genesis block.");
        Height(len - 1)
    }

    /// Returns the `state_hash` table for core tables.
    pub fn core_state_hash(&self) -> Vec<Hash> {
        // TODO: add config merkle root
        vec![
            self.transaction_results().merkle_root(),
        ]
    }
}


impl<'a> Schema<&'a mut Fork> {
    /// Mutable reference to the [`transactions`][1] index.
    ///
    /// [1]: struct.Schema.html#method.transactions
    pub(crate) fn transactions_mut(&mut self) -> MapIndex<&mut Fork, Hash, RawMessage> {
        MapIndex::new(TRANSACTIONS, self.view)
    }

    /// Mutable reference to the [`transaction_results`][1] index.
    ///
    /// [1]: struct.Schema.html#method.transaction_results
    pub(crate) fn transaction_results_mut(
        &mut self,
    ) -> ProofMapIndex<&mut Fork, Hash, TransactionResult> {
        ProofMapIndex::new(TRANSACTION_RESULTS, self.view)
    }

    /// Mutable reference to the [`transactions_pool`][1] index.
    ///
    /// [1]: struct.Schema.html#method.transactions_pool
    fn transactions_pool_mut(&mut self) -> KeySetIndex<&mut Fork, Hash> {
        KeySetIndex::new(TRANSACTIONS_POOL, self.view)
    }

    /// Mutable reference to the [`transactions_locations`][1] index.
    ///
    /// [1]: struct.Schema.html#method.transactions_locations
    pub(crate) fn transactions_locations_mut(&mut self) -> MapIndex<&mut Fork, Hash, TxLocation> {
        MapIndex::new(TRANSACTIONS_LOCATIONS, self.view)
    }

    /// Mutable reference to the [`blocks][1] index.
    ///
    /// [1]: struct.Schema.html#method.blocks
    pub(crate) fn blocks_mut(&mut self) -> MapIndex<&mut Fork, Hash, Block> {
        MapIndex::new(BLOCKS, self.view)
    }

    /// Mutable reference to the [`block_hashes_by_height_mut`][1] index.
    ///
    /// [1]: struct.Schema.html#method.block_hashes_by_height_mut
    pub(crate) fn block_hashes_by_height_mut(&mut self) -> ListIndex<&mut Fork, Hash> {
        ListIndex::new(BLOCK_HASHES_BY_HEIGHT, self.view)
    }

    /// Mutable reference to the [`block_transactions`][1] index.
    ///
    /// [1]: struct.Schema.html#method.block_transactions
    pub(crate) fn block_transactions_mut(
        &mut self,
        height: Height,
    ) -> ProofListIndex<&mut Fork, Hash> {
        let height: u64 = height.into();
        ProofListIndex::new_in_family(BLOCK_TRANSACTIONS, &height, self.view)
    }

    /// Mutable reference to the [`state_hash_aggregator`][1] index.
    ///
    /// [1]: struct.Schema.html#method.state_hash_aggregator
    pub(crate) fn state_hash_aggregator_mut(&mut self) -> ProofMapIndex<&mut Fork, Hash, Hash> {
        ProofMapIndex::new(STATE_HASH_AGGREGATOR, self.view)
    }

    /// Mutable reference to the [`consensus_messages_cache`][1] index.
    ///
    /// [1]: struct.Schema.html#method.consensus_messages
    pub(crate) fn consensus_messages_cache_mut(&mut self) -> ListIndex<&mut Fork, RawMessage> {
        ListIndex::new(CONSENSUS_MESSAGES_CACHE, self.view)
    }

    /// Saves the given consensus round value into storage.
    pub(crate) fn set_consensus_round(&mut self, round: Round) {
        let mut entry: Entry<&mut Fork, _> = Entry::new(CONSENSUS_ROUND, self.view);
        entry.set(round);
    }


    /// Adds transaction into persistent pool.
    #[doc(hidden)]
    pub fn add_transaction_into_pool(&mut self, tx: RawMessage) {
        self.transactions_pool_mut().insert(tx.hash());
        self.transactions_mut().put(&tx.hash(), tx);
    }

    /// Changes transaction status from `in_pool`, to `committed`.
    pub(crate) fn commit_transaction(&mut self, hash: &Hash) {
        self.transactions_pool_mut().remove(hash)
    }

    /// Remove transaction from persistent pool.
    #[doc(hidden)]
    pub fn reject_transaction(&mut self, hash: &Hash) -> Result<(), ()> {
        let contains = self.transactions_pool_mut().contains(hash);
        self.transactions_pool_mut().remove(hash);
        self.transactions_mut().remove(hash);
        if contains {
            Ok(())
        } else {
            Err(())
        }
    }
}

//
//#[cfg(test)]
//mod tests {
//    use rand::{thread_rng, Rng};
//    use super::*;
//    use super::super::{Database, MemoryDB};
//
//    #[test]
//    fn test_schema_fork(){
//
//    }
//}
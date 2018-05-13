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
    pub fn new(snapshot: T) -> Schema<T> {Schema{view:snapshot}}

    /// Returns table represents a map from transaction hash into raw transaction message.
    pub fn transactions(&self) -> MapIndex<&T, Hash, RawMessage> {
        MapIndex::new(TRANSACTIONS, &self.view)
    }
}
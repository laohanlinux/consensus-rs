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

use crypto::Hash;
use helpers::{Height, ValidatorId};

/// Current core information schema version.
pub const SCHEMA_MAJOR_VERSION: u16 = 0;


encoding_struct!(
    /// Exonum block header data structure.
    ///
    /// Block is essentially a list of transactions, which is
    /// a result of the consensus algorithm (thus authenticated by the supermajority of validators)
    /// and is applied atomically to the blockchain state.
    ///
    /// Header only contains the amount of transactions and the transactions root hash as well as
    /// other information, but not the transactions themselves.
    struct Block {
        /// Information schema version.
        schema_version: u16,
        /// Identifier of the block proposer.
        proposer_id: ValidatorId,
        /// Height of the block.
        height: Height,
        /// Number of transactions in block.
        tx_count: u32,
        /// Hash link to the previous block in blockchain.
        prev_hash: &Hash,
        /// Root hash of the Merkle tree of transactions in this block.
        tx_hash: &Hash,
        /// Hash of the blockchain state after applying transactions in the block.
        state_hash: &Hash,
    }
);

/// Block with pre-commits
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockProof{
    /// Block.
    pub block: Block,
    // TODO: add precommits
}
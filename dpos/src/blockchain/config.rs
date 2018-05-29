use serde::de::Error;
use serde_json::{self, Error as JsonError};

use std::collections::{BTreeMap, HashSet};

use storage::StorageValue;
use crypto::{hash, CryptoHash, Hash, PublicKey};
use helpers::{Height, Milliseconds};

/// Consensus algorithm parameters
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConsensusConfig{
    /// Maximum number of transactions per block.
    pub txs_block_limit: u32,
    /// Maximum message length (in bytes).
    pub max_message_len: u32,
}

impl ConsensusConfig {
    /// Default value for max_message_len.
    pub const DEFAULT_MAX_MESSAGE_LEN: u32 = 1024;
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            txs_block_limit: 200,
            max_message_len: Self::DEFAULT_MAX_MESSAGE_LEN,
        }
    }
}
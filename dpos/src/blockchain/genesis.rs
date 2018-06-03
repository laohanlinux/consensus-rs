use super::ConsensusConfig;

use time;
use chrono::*;

use consensue::{slot, delegates};
use helpers::Height;

/// The initial `exonum-core` configuration which is committed into the genesis block.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GenesisConfig {
    /// Configuration of consensus.
    pub consensus: ConsensusConfig,
    /// Genesis block timestamp
    pub genesis_timestamp: i64,
}

impl GenesisConfig {
    pub const GENESIS_TIME:&'static str = "2017-12-19T16:39:57-08:00";
    pub fn new() -> Self {
        let genesis_time = DateTime::parse_from_rfc3339(Self::GENESIS_TIME).unwrap();
        Self::new_with_consensus(genesis_time.timestamp(), ConsensusConfig::default())
    }

    /// Creates a configuration from the given consensus configuration and list public keys.
    pub fn new_with_consensus(timestamp: i64, consensus: ConsensusConfig) -> Self {
        GenesisConfig {
            consensus,
            genesis_timestamp: timestamp,
        }
    }
}
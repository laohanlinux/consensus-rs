use super::config::ConsensusConfig;

/// The initial `exonum-core` configuration which is committed into the genesis block.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GenesisConfig {
    /// Configuration of consensus.
    pub consensus: ConsensusConfig,
}

impl GenesisConfig {
    pub fn new() -> Self {
        Self::new_with_consensus(ConsensusConfig::default())
    }

    /// Creates a configuration from the given consensus configuration and list public keys.
    pub fn new_with_consensus(consensus: ConsensusConfig) -> Self {
        GenesisConfig {
            consensus,
        }
    }
}
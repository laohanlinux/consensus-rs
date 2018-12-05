use std::time::Duration;
use std::collections::HashMap;

use toml::Value as Toml;
use toml::value::Table;
use toml::value::Datetime;

use crate::common::random_dir;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub ip: String,
    pub port: u16,
    #[serde(with = "serde_millis")]
    pub block_period: Duration,
    #[serde(with = "serde_millis")]
    pub request_time: Duration,
    pub peer_id: String,
    #[serde(with = "serde_millis")]
    pub ttl: Duration,
    pub store: String,
    pub genesis: Option<GenesisConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GenesisConfig {
    pub block_hash: String,
    pub validator: Vec<String>,
    pub accounts: Table,
    pub epoch_time: Datetime,
    pub proposer: String,
    pub gas_used: u64,
    pub extra: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ip: "127.0.0.1".to_string(),
            port: 7960,
            block_period: Duration::from_millis(3 * 1000),
            request_time: Duration::from_millis(3 * 1000),
            peer_id: "QmbBr2fHwLFKvHkAq1BpbEr4dvR8P6orQxHkVaxeJsJiW8".to_string(),
            ttl: Duration::from_millis(5 * 1000),
            store: *random_dir(),
            genesis: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;
    use std::str::FromStr;

    #[test]
    fn t_config() {
        println!("{:?}", PeerId::random());
        println!("{:?}", PeerId::from_str("QmbBr2fHwLFKvHkAq1BpbEr4dvR8P6orQxHkVaxeJsJiW8").unwrap());
    }
}
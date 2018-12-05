use std::sync::Arc;
use std::str::FromStr;
use std::str::Utf8Error;

use serde::{Serialize, Serializer, Deserialize, Deserializer};
use ethereum_types::H160;
use parking_lot::RwLock;

use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::EMPTY_HASH;

use crate::{
    types::{Timestamp, Gas, Difficulty, Height, EMPTY_ADDRESS},
    types::block::{Block, Header},
    config::GenesisConfig,
    common,
};
use super::{
    ledger::Ledger,
};

pub(crate) fn store_genesis_block(genesis_config: &GenesisConfig, ledger: Arc<RwLock<Ledger>>) -> Result<(), String> {
    use chrono::{Local, DateTime, ParseError};
    let mut ledger = ledger.write();
    if ledger.get_genesis_block().is_some() {
        ledger.load_genesis();
        return Ok(());
    }
    // TODO Add more xin
    let proposer = common::string_to_address(&genesis_config.proposer)?;
    let epoch_time: DateTime<Local> = {
        let epoch_time_str = genesis_config.epoch_time.to_string();
        DateTime::from_str(&epoch_time_str)
    }.map_err(|err: ParseError| err.to_string())?;

    let extra = genesis_config.extra.as_bytes().to_vec();
    let header = Header::new(EMPTY_HASH, proposer, EMPTY_HASH, EMPTY_HASH, EMPTY_HASH,
                             0, 0, 0, genesis_config.gas_used + 10, genesis_config.gas_used,
                             epoch_time.timestamp() as Timestamp, None, Some(extra));
    let block = Block::new(header, vec![]);
    ledger.add_genesis_block(&block);
    ledger.load_genesis();
    Ok(())
}
use chrono::prelude::*;
use util_rs;
use bigint::U256;

pub const VERSION: u32 = 1;
pub const VRF_SIZE: usize = 64;
pub const MAX_PROPOSER_COUNT: isize = 32;
pub const MAX_ENDORSER_COUNT: isize = 240;
pub const MAX_COMMITTER_COUNT: isize = 240;

#[derive(Serialize, Deserialize, Clone)]
pub struct PeerConfig {
    index: u32,
    ID: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChainConfig {
    // software version
    #[serde(rename = "version")]
    version: u32,
    // config-updated version
    view: u32,
    //  network size
    n: u32,
    //  consensus quorom
    c: u32,
    // block msg deplay
    block_meg_delay: DateTime<Utc>,
    // hash msg deplay
    hash_msg_delay: DateTime<Utc>,
    // peer handshake timeout
    peer_handshake_timeout: DateTime<Utc>,
    peers: Vec<PeerConfig>,
    pos_table: Vec<u32>,
    max_block_change_view: u32,
}

impl ChainConfig {
    pub fn hash(&self) {

    }
}

///
/// VBFT consensus payload, store on each block header
///
#[derive(Serialize, Deserialize, Clone)]
pub struct VbfBlockInfo {
    #[serde(rename = "leader")]
    proposer: u32,
    vrf_value: Vec<u8>,
    vrf_proof: Vec<u8>,
    last_config_block_num: u32,
    new_chain_config: ChainConfig,
}

struct VRFValue([u8; VRF_SIZE]);

impl VRFValue {
    pub fn new() -> VRFValue {
        VRFValue([0; VRF_SIZE])
    }
    pub fn bytes(&self) -> Vec<u8> {
       self.0.to_vec()
    }
    pub fn from_bytes(&mut self, b: &[u8]) -> Result<(), ()> {
        assert_eq!(b.len(), VRF_SIZE);
        self.0.iter_mut().fold(0, |acc, mut item| {
           *item = b[acc];
            acc+1
        });
        Ok(())
    }
    pub fn is_nil(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }
}

use std::fmt::{self, Formatter, Display};
use hex;
impl Display for VRFValue {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), fmt::Error> {
        let hex_str = hex::encode_upper(&self.bytes());
        writeln!(f, "{}", hex_str).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod test{
    #[test]
    fn test_vrf_value(){
        assert!(super::VRFValue::new().is_nil());
        let mut vrf1 = super::VRFValue::new();
        let mut vrf2    = vrf1.bytes();
    }
}

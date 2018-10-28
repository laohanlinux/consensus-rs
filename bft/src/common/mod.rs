use sha3::{Sha3_256, Digest};
use bigint::U256;
use rand::random;

use std::fmt::{self, Display};
use std::env;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HexBytes {
    inner: [u8; 32],
}

impl HexBytes {
    pub fn bytes(&self) -> &[u8;32] {
        &self.inner
    }

    pub fn string(&self) -> String {
        String::from_utf8_lossy(&self.inner).to_string()
    }
}

impl Display for HexBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use std::fmt;
        writeln!(f, "{}", self.string()).unwrap();
        Ok(())
    }
}

pub fn as_256(data: &[u8]) -> U256 {
    U256::from_big_endian(data)
}

pub fn u256_hash(input: &[u8]) -> Vec<u8>{
    let mut hasher = Sha3_256::default();
    hasher.input(input);
    hasher.result().to_vec()
}

pub fn random_dir() -> Box<String> {
    Box::new(format!("{}{}", env::temp_dir().to_str().unwrap(), random::<u64>()))
}
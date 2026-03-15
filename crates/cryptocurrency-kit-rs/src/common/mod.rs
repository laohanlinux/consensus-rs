use std::io::Cursor;

use ethereum_types::H256;
use hex;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize, Serializer as stdSer};
use sha3::{Digest, Sha3_256};
use crate::ethkey::keccak::Keccak256;

//TODO Opz
pub fn to_fixed_array_20(data: &[u8]) -> [u8; 20] {
    let mut buffer = [0u8; 20];
    buffer.clone_from_slice(&data[0..20]);
    buffer
}

pub fn to_fixed_array_32(data: &[u8]) -> [u8; 32] {
    let mut buffer = [0u8; 32];
    buffer.clone_from_slice(&data[0..32]);
    buffer
}

pub fn to_fixed_array_64(data: &[u8]) -> [u8; 64] {
    let mut buffer = [0u8; 64];
    buffer.clone_from_slice(&data[0..64]);
    buffer
}

pub fn to_fixed_array(data: &[u8], fix_len: usize) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.clone_from_slice(&data[0..fix_len]);
    buffer
}

pub fn to_hex<T: AsRef<[u8]>>(data: T) -> String {
    hex::encode(data)
}

pub fn from_hex<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(data)
}

pub fn to_sha3(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::default();
    hasher.input(data);
    hasher.result().to_vec()
}

pub fn to_keccak<T: AsRef<[u8]>>(data: T) -> H256 {
    data.as_ref().keccak256().into()
}

pub fn to_msgpack_vec<T: stdSer + Serialize>(obj: T, buf: &mut [u8]) {
    obj.serialize(&mut Serializer::new(buf)).unwrap();
}

pub fn from_msgpack<T: 'static + stdSer + Deserialize<'static> + Serialize>(buf: &[u8]) -> T {
    let cur = Cursor::new(buf);
    let mut de = Deserializer::new(cur);
    Deserialize::deserialize(&mut de).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    #[test]
    fn keccak_sha() {
        let v = vec![1, 2, 3];
        let sha3 = to_sha3(&v);
        let keccak = to_keccak(&v);
        writeln!(io::stdout(), "{:?}", sha3).unwrap();
        writeln!(io::stdout(), "{:?}", &keccak[0..32]).unwrap();
    }
}
use sha3::{Digest, Sha3_256};
use std::iter::FromIterator;
use std::string::String;

use crate::common;

use rustc_hex::{FromHexError, ToHex};
use crate::ethkey::{Message, Error as SignError, signature::sign, Signature};
use crate::ethkey::secret::Secret;

pub const HASH_SIZE: usize = 32;
pub const EMPTY_HASH: Hash = Hash([0_u8; HASH_SIZE]);

pub fn empty_hash(hash: &Hash) -> bool {
    hash.0 == EMPTY_HASH.0
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Hash([u8; HASH_SIZE]);

impl Hash {
    // Create a new instance from bytes array.
    pub fn new(b: &[u8]) -> Self {
        let size = b.len();
        assert_eq!(size, HASH_SIZE);
        let mut buf = [0; 32];
        for item in b.iter().enumerate() {
            buf[item.0] = *item.1;
        }
        Hash(buf)
    }

    /// Create a new instance from bytes slice
    pub fn from_slice(bs: &[u8]) -> Option<Self> {
        let size = bs.len();
        assert_eq!(size, HASH_SIZE);
        Some(Self::new(bs))
    }

    pub fn sign(&self, secret: &Secret) -> Result<Signature, SignError> {
        let message = Message::from_slice(&self.0);
        sign(secret, &message)
    }

    /// Create a new install with filled with zeros.
    pub fn zero() -> Self {
        Hash([0; HASH_SIZE])
    }

    pub fn to_hex(&self) -> String {
        common::to_hex(self)
    }

    pub fn short(&self) -> String {
        let hex = self.to_hex();
        format!("0x{}...{}", &hex[..6], &hex[hex.len() - 6..]).to_string()
    }
}

/// It is very good
impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

use std::str::FromStr;

impl FromStr for Hash {
    type Err = FromHexError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == HASH_SIZE {
            let out: Vec<u8> = s.chars().map(|c| c as u8).collect();
            return Ok(Hash::new(&out));
        } else if s.len() == (HASH_SIZE + 2) {
            let out: Vec<u8> = s.chars().skip(2).map(|c| c as u8).collect();
            return Ok(Hash::new(&out));
        } else {
            return Err(FromHexError::InvalidHexLength);
        }
    }
}

use std::fmt;

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

impl ToHex for Hash {
    fn to_hex<T: FromIterator<char>>(&self) -> T {
        self.as_ref().to_hex()
    }
}

pub trait CryptoHash {
    fn hash(&self) -> Hash;
}

impl Default for Hash {
    fn default() -> Hash {
        Hash::zero()
    }
}

impl fmt::LowerHex for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{}", self.to_hex().to_lowercase())
    }
}

use std::hash::Hash as stdHash;
use std::hash::Hasher as stdHasher;

impl stdHash for Hash {
    fn hash<H: stdHasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

pub fn hash<T: AsRef<[u8]>>(data: T) -> Hash {
    let digest = common::to_keccak(data);
    Hash::new(digest.as_ref())
}

#[derive(Debug, Default)]
pub struct HashStream(Sha3_256);

impl HashStream {
    /// Create a new instance of `HashStream`
    pub fn new() -> Self {
        HashStream(Sha3_256::default())
    }

    /// Processes a chunk of stream and returns a `HashStream` with the updated internal state.
    pub fn update(mut self, chunk: &[u8]) -> Self {
        self.0.input(chunk);
        self
    }

    /// Returns the hash of data supplied to the stream so far.
    pub fn hash(self) -> Hash {
        let dig = self.0.result().to_vec();
        Hash::new(&dig)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::{self, Write};

//    impl CryptoHash for Vec<u8> {
//        fn hash(&self) -> Hash {
//            let mut buf = Vec::new();
//            self.serialize(&mut Serializer::new(&mut buf)).unwrap();
//            hash(&buf)
//        }
//    }


    #[test]
    fn hash() {
        for i in 0..100 {
            writeln!(io::stdout(), "{:#?}", super::hash(vec![i])).unwrap();
        }
        let hash = CryptoHash::hash(&vec![10]);
        writeln!(io::stdout(), "-->{:#?}", hash).unwrap();
    }

    #[test]
    fn t_empty_hash() {
        let h1 = Hash([0_u8; HASH_SIZE]);
        assert!(empty_hash(&h1));
        assert!(!empty_hash(&Hash([1_u8; HASH_SIZE])));
        assert_eq!(EMPTY_HASH, Hash([0_u8; HASH_SIZE]));
    }

    #[test]
    fn t_short_term() {
        for i in 0..100 {
            writeln!(io::stdout(), "{:#?}", super::hash(vec![i]).short()).unwrap();
        }
    }

    #[test]
    fn hash_eq() {
        let h1: Hash = CryptoHash::hash(&vec![1, 2]);
        let h2: Hash = CryptoHash::hash(&vec![1, 2]);
        println!("h1:{:?}, h2:{:?}", h1, h2);
        let eq = h1 == h2;
        assert!(eq);
        assert_eq!(h1, h2);
    }
}

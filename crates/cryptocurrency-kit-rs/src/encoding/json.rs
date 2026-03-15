use std::string::String;
use std::time::Duration;

use crate::types::Zero;
use crate::crypto::*;
use crate::ethkey::Public as PublicKey;

use chrono::prelude::*;
use uuid::Uuid;

/////////////////////////////////////////////
#[macro_export]
macro_rules! implement_cryptohash_traits {
    ($key: ident) => {
        impl CryptoHash for $key {
            fn hash(&self) -> Hash {
                let buf = serde_json::to_vec(self).unwrap();
                hash(&buf)
            }
        }
    };
}

implement_cryptohash_traits! {bool}
implement_cryptohash_traits! {u8}
implement_cryptohash_traits! {u16}
implement_cryptohash_traits! {u32}
implement_cryptohash_traits! {u64}
implement_cryptohash_traits! {i8}
implement_cryptohash_traits! {i16}
implement_cryptohash_traits! {i32}
implement_cryptohash_traits! {i64}
implement_cryptohash_traits! {String}
implement_cryptohash_traits! {Uuid}
implement_cryptohash_traits! {Duration}
implement_cryptohash_traits! {PublicKey}

impl CryptoHash for () {
    fn hash(&self) -> Hash {
        let buf = serde_json::to_vec(self).unwrap();
        hash(&buf)
    }
}

impl CryptoHash for Zero {
    fn hash(&self) -> Hash {
        EMPTY_HASH
    }
}

impl CryptoHash for Vec<u8> {
    fn hash(&self) -> Hash {
//        let mut buf = Vec::new();
//        self.serialize(&mut Serializer::new(&mut buf)).unwrap();
//        hash(&buf)
        hash(self)
    }
}

impl CryptoHash for DateTime<Utc> {
    fn hash(&self) -> Hash {
        let buf = serde_json::to_vec(self).unwrap();
        hash(&buf)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ethkey::Generator;
    use std::io::{self, Write};

    #[test]
    fn u8_hsh() {
        let u_8: u8 = u8::from(100);
        writeln!(io::stdout(), "u8_hash {:?}", u_8.hash()).unwrap();
    }

    #[test]
    fn bool_hash() {
        writeln!(io::stdout(), "bool_true_hash {:?}", true.hash()).unwrap();
        writeln!(io::stdout(), "bool_false_hash {:?}", false.hash()).unwrap();
    }

    #[test]
    fn i8_hash() {
        writeln!(io::stdout(), "i8_hash {:?}", i8::from(100).hash()).unwrap();
    }

    #[test]
    fn publickey_hash() {
        (0..100).for_each(|_i| {
            let keypair = crate::ethkey::Random {}.generate().unwrap();
            let pubkey = keypair.public();
            let buf = serde_json::to_vec(pubkey).unwrap();
            writeln!(io::stdout(), "{}", buf.len()).unwrap();
        })
    }

    #[test]
    fn vec_hash() {
        let buf = serde_json::to_vec(&vec![0, 0]).unwrap();
        writeln!(io::stdout(), "{}", buf.len()).unwrap();
    }

    #[test]
    fn json_unit() {
        let buf = serde_json::to_vec(&()).unwrap();
        writeln!(io::stdout(), "{:?}", buf).unwrap();
    }

    #[test]
    fn batch() {
        for i in 0..(2 << 10) {
            writeln!(io::stdout(), "random_{} {:?}", i, i.hash()).unwrap();
        }
    }

    #[test]
    fn de_vec() {
        use crate::common::to_keccak;
        let v = vec![1];
        println!("{:?}", v.hash());
        let digest = to_keccak(v);
        println!("{:?}", digest);
    }
}

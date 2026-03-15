use secp256k1;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Err code: {}", _0)]
    Secp(secp256k1::Error),
    #[fail(display = "Err code: {}", _0)]
    Io(::std::io::Error),
    #[fail(display = "An error has occurred.")]
    InvalidMessage,
    #[fail(display = "symm error has occurred.")]
    Symm,
}

pub mod sign {
    
    use crate::crypto::Hash;
    use crate::ethkey::{Public, Secret, SECP256K1};
    use secp256k1::{self, key, Message, Signature};

    pub fn verify(public: &Public, sign: &Signature, plain_text_hash: &Hash) -> bool {
        let context = &SECP256K1;
        // the first byte flag whether compress
        let pdata = {
            let mut temp = [4u8; 65];
            (&mut temp[1..65]).copy_from_slice(&public[0..64]);
            temp
        };
        let publ = key::PublicKey::from_slice(context, &pdata).unwrap();
        context
            .verify(
                &Message::from_slice(plain_text_hash.as_ref()).unwrap(),
                &sign,
                &publ,
            )
            .is_ok()
    }

    pub fn sign(message: &Message, secret: &Secret) -> Signature {
        let context = &SECP256K1;
        context
            .sign(message, &secret.to_secp256k1_secret().unwrap())
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::crypto::{hash, CryptoHash, Hash};
    use crate::encoding::json::*;
    use ethereum_types::H256;
    use crate::ethkey::random::Random;
    use crate::ethkey::Generator;
    use crate::ethkey::{Address, KeyPair, Public};
    use serde::{Deserialize, Serialize};
    use std::io::{self, Write};

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Block {
        height: u64,
        validator: Vec<Validator>,
    }

    implement_cryptohash_traits! {Block}

    impl Block {
        fn new(height: u64, validator: Vec<Validator>) -> Block {
            Block { height, validator }
        }
    }

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Validator {
        address: Address,
        publickey: Public,
    }

    impl Validator {
        fn new(keypair: &KeyPair) -> Validator {
            let publickey = keypair.public();
            let address = keypair.address();
            Validator {
                address,
                publickey: *publickey,
            }
        }
    }

    #[test]
    fn error() {
        writeln!(io::stdout(), "{:?}", Error::Symm).unwrap();
    }

    #[test]
    fn sign() {
        (0..100).for_each(|i| {
            let keypair = Random::generate_keypair();
            let val = Validator::new(&keypair);
            let block = Block::new(i as u64, vec![val]);
            let hash = block.hash();
            writeln!(io::stdout(), "{}: {}", i, crate::common::to_hex(hash.as_ref())).unwrap();
            let secp_hash = secp256k1::Message::from_slice(hash.as_ref()).unwrap();
            let signature = sign::sign(&secp_hash, keypair.secret());

            // verify signature
            let ok = sign::verify(keypair.public(), &signature, &hash);
            assert!(ok);
        })
    }
}

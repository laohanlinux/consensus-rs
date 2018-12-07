use byteorder::WriteBytesExt;
use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash, HASH_SIZE};
use cryptocurrency_kit::ethkey::Secret;
use cryptocurrency_kit::ethkey::{public_to_address, recover_bytes};
use cryptocurrency_kit::ethkey::{Address, Public, Signature};

use crate::protocol::{GossipMessage, MessageType};

const SIGN_OP_OFFSET: usize = 0;
const SIGN_ROUND_OFFSET: usize = 1;
const SIGN_PACKET_SIZE: usize = 9;

use std::io::Cursor;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Votes(Vec<Signature>);

impl Votes {
    pub fn new(votes: Vec<Signature>) -> Self {
        Votes(votes)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn add_votes(&mut self, votes: &Vec<Signature>) {
        for vote in votes {
            self.add_vote(vote);
        }
    }

    pub fn add_vote(&mut self, vote: &Signature) -> bool {
        let ok = self.0.iter().any(|e_vote| *e_vote == *vote);
        if ok {
            return ok;
        }
        self.0.push(vote.clone());
        false
    }

    pub fn remove_vote(&mut self, vote: &Signature) -> bool {
        self.0.remove_item(&vote).is_some()
    }

    pub fn votes(&self) -> &Vec<Signature> {
        &self.0
    }

    pub fn verify_signs<F>(&self, digest: Hash, author: F) -> bool
        where
            F: Fn(Address) -> bool,
    {
        self.0.iter().all(
            |signature| {
                match decrypt_commit_bytes(digest.as_ref(), &signature) {
                    Ok(address) => {
                        author(address)
                    }
                    Err(_) => return false,
                }
            },
        )
    }
}

pub fn decrypt_commit_bytes<T: AsRef<[u8]>>(
    input: T,
    signture: &Signature,
) -> Result<Address, String> {
    let input = input.as_ref();
    if input.len() != SIGN_PACKET_SIZE {
        return Err("sign bytes size not equal SIGN_PACKET_SIZE".to_string());
    }
    let op_code = MessageType::Commit;
    let digest = hash(input);
    let mut input = Cursor::new(vec![0_u8; 1 + HASH_SIZE]);
    input.write_u8(op_code as u8).unwrap();
    input.write_all(digest.as_ref()).unwrap();
    let buffer = input.into_inner();
    let digest: Hash = hash(buffer);
    match recover_bytes(signture, digest.as_ref()) {
        Ok(ref public) => {
            let address = public_to_address(public);
            return Ok(address);
        }
        Err(_) => {
            return Err("recover commit sign failed".to_string());
        }
    }
}

pub fn encrypt_commit_bytes(digest: &Hash, secret: &Secret) -> Signature {
    let mut input = Cursor::new(vec![0_u8; 1 + HASH_SIZE]);
    input.write_u8(MessageType::Commit as u8).unwrap();
    input.write_all(digest.as_ref()).unwrap();
    let buffer = input.into_inner();
    let digest = hash(buffer);
    digest.sign(secret).unwrap()
}


#[cfg(test)]
mod tests{
    use super::*;
    use cryptocurrency_kit::ethkey::KeyPair;
    use cryptocurrency_kit::ethkey::Generator;
    use cryptocurrency_kit::ethkey::Random;

    #[test]
    fn t_random() {
        (0..10).for_each(|_|{
            let keypair = Random{}.generate().unwrap();
            println!("{:?}, {:?}",  keypair, keypair.address());
        });
    }
}
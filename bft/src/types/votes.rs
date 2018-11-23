use cryptocurrency_kit::ethkey::{Address, Public, Signature};
use cryptocurrency_kit::ethkey::{public_to_address, recover_bytes};
use cryptocurrency_kit::ethkey::Secret;
use cryptocurrency_kit::crypto::Hash;

use crate::{
    protocol::{MessageType, GossipMessage},

};

const SIGN_OP_OFFSET: usize = 0;
const SIGN_ROUND_OFFSET: usize = 1;
const SIGN_PACKET_SIZE: usize = 9;

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
        where F: Fn(Address) -> bool
    {
        self.0.iter().all(|signature| {
            match recover_bytes(signature, digest.as_ref()) {
                Ok(ref public) => {
                    let address = public_to_address(public);
                    author(address)
                }
                Err(_) => false,
            }
        })
    }
}

//
//pub fn decrypt_commit_bytes<T: AsRef<&[u8]>>(input: T, signture: &Signature) -> Result<Signature, String> {
//    if input.as_ref().len() != SIGN_PACKET_SIZE {
//        return Err("sign bytes size not equal SIGN_PACKET_SIZE".to_string());
//    }
//    let op_code = MessageType::Commit;
//    let digest = header.hash();
//    let mut input = Cursor::new(vec![0_u8; 1 + HASH_SIZE]);
//    input.write_u8(op_code as u8).unwrap();
//    input.write(digest.as_ref()).unwrap();
//    let buffer = input.into_inner();
//    let digest: Hash = hash(buffer);
//    if votes.verify_signs(digest, |validator| { self.validator_set.get_by_address(validator).is_some() }) == false {
//        return Err("invalid votes".to_string());
//    }
//    let maj32 = self.validator_set.two_thirds_majority();
//    if maj32 + 1 > votes.len() {
//        return Err("lack votes".to_string());
//    }
//    Ok(())
//}
//
//pub fn encrypt_commit_bytes(digest: &Hash, secret: &Secret) {}
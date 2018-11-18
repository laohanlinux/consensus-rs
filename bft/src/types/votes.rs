use cryptocurrency_kit::ethkey::{Address, Public, Signature};
use cryptocurrency_kit::ethkey::{public_to_address, recover_bytes};
use cryptocurrency_kit::crypto::Hash;

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
        self.0.iter().all(|signature|{
            match recover_bytes(signature, digest.as_ref()) {
                Ok(ref public) => {
                    let address = public_to_address(public);
                    author(address)
                },
                Err(_) => false,
            }
        })
    }
}

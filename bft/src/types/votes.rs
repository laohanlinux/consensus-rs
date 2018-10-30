use cryptocurrency_kit::ethkey::signature::Signature;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Votes(Vec<Signature>);

impl Votes {
    pub fn new(votes: Vec<Signature>) -> Self {
        Votes(votes)
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
}

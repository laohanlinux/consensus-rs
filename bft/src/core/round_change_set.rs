use std::collections::HashMap;

use crate::{
    protocol::{MessageManage, GossipMessage},
    consensus::types::{Round, View},
    consensus::validator::{ValidatorSet, Validators, ImplValidatorSet},
};

pub struct RoundChangeSet<V: ValidatorSet> {
    validator_set: V,
    round_changes: HashMap<u64, MessageManage>,
    pre_round_change_sign: Vec<Vec<u8>>, // 暂时未用
}

impl RoundChangeSet<ImplValidatorSet> {
    pub fn add(&mut self, round: Round, msg: GossipMessage) -> Result<usize, String> {
        let val_set = self.validator_set.clone();
        let mut msg_manager = self.round_changes.entry(round).or_insert_with(||{
            MessageManage::new(View::default(), val_set)
        });
        msg_manager.add(msg).map(|_|{0})?;
        Ok(msg_manager.len())
    }

    pub fn round_change_set(&self, round: &Round) -> Option<&MessageManage> {
        self.round_changes.get(round)
    }

    // TODO
//    pub fn pre_round_change_bytes(&self)

//    pub fn max_round_change_changes_bytes(&self, n: usize) -> (Round, &Vec<Vec<u8>>) {
//
//    }
}
use std::collections::HashMap;

use crate::{
    protocol::{MessageManage, GossipMessage},
    consensus::types::{Round, View},
    consensus::validator::{ValidatorSet, Validators, ImplValidatorSet},
};

type PreRCSignBytes = Vec<Vec<u8>>;

pub struct RoundChangeSet<V: ValidatorSet> {
    validator_set: V, // 当前所有轮次的validators
    round_changes: HashMap<u64, MessageManage>,
    // 每个轮次的消息管理器
    pre_rc_sign_bytes: Option<PreRCSignBytes>, // 暂时未用
}

impl RoundChangeSet<ImplValidatorSet> {
    pub fn new(validators: ImplValidatorSet, pre_rc_sign_bytes: Option<PreRCSignBytes>) -> RoundChangeSet<ImplValidatorSet> {
        RoundChangeSet {
            validator_set: validators,
            round_changes: HashMap::new(),
            pre_rc_sign_bytes: pre_rc_sign_bytes,
        }
    }

    pub fn add(&mut self, round: Round, msg: GossipMessage) -> Result<usize, String> {
        let val_set = self.validator_set.clone();
        let msg_manager = self.round_changes.entry(round).or_insert_with(|| {
            MessageManage::new(View::default(), val_set)
        });
        msg_manager.add(msg).map(|_| { 0 })?;
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
    pub fn clear(&mut self, round: Round) {
        // dereference
        self.round_changes.retain(|&round_, mm| {
            mm.len() > 0 && round > round_
        });
    }

    // return the max round which the number of messages is equal or larger than num
    pub fn max_round(&self, num: usize) -> Option<Round> {
        if let Some((round, _)) = self.round_changes.iter().max_by(|x, y| x.0.cmp(y.0)) {
            if self.round_changes.get(round).unwrap().len() >= num {
                return Some(*round);
            }
        }
        None
    }

    pub fn print_info(&self) {
        for round_change in &self.round_changes {
            debug!("round:{:?}, size:{:?}", round_change.0, round_change.1.len());
        }
    }
}
use cryptocurrency_kit::crypto::{CryptoHash, Hash, EMPTY_HASH};
use cryptocurrency_kit::storage::values::StorageValue;

use crate::{
    consensus::validator::{ValidatorSet, ImplValidatorSet},
    consensus::types::{PrePrepare, Proposal, Request, Round, Subject, View},
    protocol::{GossipMessage, MessageManage, MessageType},
    types::Height,
};

// it is not safe
pub struct RoundState<T: CryptoHash + StorageValue> {
    round: Round,
    height: Height,
    preprepare: Option<PrePrepare>,
    // 提案
    prepares: MessageManage,
    commits: MessageManage,
    pending_request: Option<Request<T>>,
    lock_hash: Option<Hash>, // 锁hash
}

impl<T> RoundState<T>
    where
        T: CryptoHash + StorageValue,
{
//    pub(crate) fn new_round_state<V: ValidatorSet>(view: View, vals: V,
//                                                   lock_hash: Option<Hash>,
//                                                   preprepare: Option<PrePrepare>,
//                                                   pending_request: Option<Request<T>>)
//        -> Self {
//        RoundState{
//            round: view.round,
//            height: view.height,
//            preprepare: preprepare,
//            prepares: MessageManage::new(view.clone(), vals as ImplValidatorSet),
//            commits: MessageManage::new(view.clone(), vals),
//            pending_request: pending_request,
//            lock_hash: lock_hash,
//        }
//    }

    pub(crate) fn get_prepare_or_commit_size(&self) -> usize {
        let mut result = self.prepares.len() + self.commits.len();
        self.prepares.values().iter().for_each(|message| {
            if self.commits.get_message(message.address).is_some() {
                result -= 1;
            }
        });

        result
    }

    pub(crate) fn subject(&self) -> Option<Subject> {
        if self.preprepare.is_none() {
            return None;
        }
        Some(Subject {
            view: View {
                round: self.round,
                height: self.height,
            },
            digest: self.preprepare.as_ref().unwrap().proposal.block().hash(),
        })
    }

    pub(crate) fn set_preprepare(&mut self, preprepare: PrePrepare) {
        self.preprepare = Some(preprepare);
    }

    pub(crate) fn proposal(&self) -> Option<&Proposal> {
        if self.preprepare.is_none() {
            None
        } else {
            Some(&self.preprepare.as_ref().unwrap().proposal)
        }
    }

    pub(crate) fn set_round(&mut self, round: Round) {
        self.round = round;
    }

    pub(crate) fn round(&self) -> Round {
        self.round
    }

    pub(crate) fn set_height(&mut self, height: Height) {
        self.height = height;
    }

    pub(crate) fn height(&self) -> Height {
        self.height
    }

    pub(crate) fn is_locked(&self) -> bool {
        self.lock_hash.is_some()
    }

    // 锁定提案
    pub(crate) fn lock_hash(&mut self) {
        if self.preprepare.is_none() {
            return;
        }

        self.lock_hash = Some(self.preprepare.as_ref().unwrap().proposal.block().hash());
        trace!(
            "Lock proposal, hash:{}",
            self.lock_hash.as_ref().unwrap().short()
        );
    }

    // 解锁提案
    pub(crate) fn unlock_hash(&mut self) {
        trace!(
            "Unlock proposal, hash:{}",
            self.lock_hash.as_ref().or_else(|| Some(&EMPTY_HASH)).unwrap().short()
        );
        self.lock_hash = None;
    }

    pub(crate) fn get_lock_hash(&self) -> Option<Hash> {
        self.lock_hash.as_ref().cloned()
    }
}

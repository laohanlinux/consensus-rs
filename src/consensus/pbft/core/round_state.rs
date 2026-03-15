use cryptocurrency_kit::crypto::{Hash, EMPTY_HASH};

use crate::{
    consensus::validator::ImplValidatorSet,
    consensus::types::{PrePrepare, Proposal, Request, Round, Subject, View},
    protocol::MessageManage,
    types::Height,
};

// it is not safe
pub struct RoundState {
    round: Round,
    height: Height,
    // 提案
    pub preprepare: Option<PrePrepare>,
    pub prepares: MessageManage,
    pub commits: MessageManage,
    pub pending_request: Option<Request<Proposal>>,
    // 自己的提案
    lock_hash: Option<Hash>, // 锁hash
}


impl RoundState
{
    pub(crate) fn new_round_state(view: View, vals: ImplValidatorSet,
                                  lock_hash: Option<Hash>,
                                  preprepare: Option<PrePrepare>,
                                  pending_request: Option<Request<Proposal>>)
                                  -> Self {
        RoundState {
            round: view.round,
            height: view.height,
            preprepare,
            prepares: MessageManage::new(view, vals.clone()),
            commits: MessageManage::new(view, vals.clone()),
            pending_request,
            lock_hash,
        }
    }

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
        self.preprepare.as_ref()?;
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

    #[allow(dead_code)]
    pub(crate) fn set_round(&mut self, round: Round) {
        self.round = round;
    }

    pub(crate) fn round(&self) -> Round {
        self.round
    }

    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub(crate) fn unlock_hash(&mut self) {
        trace!(
            "Unlock proposal, hash:{}",
            self.lock_hash.as_ref().unwrap_or(&EMPTY_HASH).short()
        );
        self.lock_hash = None;
    }

    pub(crate) fn get_lock_hash(&self) -> Option<Hash> {
        self.lock_hash.as_ref().cloned()
    }
}

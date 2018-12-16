use std::borrow::Cow;
use std::time::Instant;
use std::time::Duration;

use cryptocurrency_kit::crypto::EMPTY_HASH;
use cryptocurrency_kit::storage::values::StorageValue;

use crate::{
    consensus::error::{ConsensusError, ConsensusResult},
    consensus::validator::ValidatorSet,
    consensus::types::{Round, Subject, View},
    protocol::{GossipMessage, MessageType},
    types::Validator,
};

use super::core::Core;

pub trait HandleRoundChange {
    fn send_next_round_change(&mut self);
    fn send_round_change(&mut self, round: Round);
    // receive a round change message and handle it
    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult;
}

impl HandleRoundChange for Core {
    fn send_next_round_change(&mut self) {
        let current_view = self.current_view();
        self.round_change_set.print_info();
        // Find the max
        let round = self.round_change_set.max_round();
        if round <= current_view.round {
            self.send_round_change(current_view.round + 1);
        } else {
            self.send_round_change(round);
        }
    }

    fn send_round_change(&mut self, round: Round) {
        if Instant::now().duration_since(self.round_change_limiter) <= Duration::from_millis(50) {
            debug!("Skip round change sent");
            self.new_round_change_timer();
            return;
        }
        self.round_change_limiter = Instant::now();

        if self.current_view().round < round {
            self.catchup_round(round);
        }
        let current_view = self.current_view();

//        let ok = current_view.round < round;
//        assert!(ok);

        // TODO add pre max round change prove
        let subject = Subject {
            view: View::new(current_view.height, round),
            digest: EMPTY_HASH,
        };
        debug!("Vote for round change, current:{}, vote: {}", current_view.round, round);
        let mut msg = GossipMessage::new(MessageType::RoundChange, subject.into_bytes(), None);
        msg.create_time = chrono::Local::now().timestamp_millis() as u64;
        self.broadcast(&msg);
    }

    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult {
        let subject: Subject = Subject::from_bytes(Cow::from(msg.msg()));
        debug!("Handle round change message from {:?}, from me: {}, subject: {:?}", src.address(), self.address() == *src.address(), subject);
        self.check_message(MessageType::RoundChange, &subject.view)?;
        let current_view = self.current_view();
        let current_val_set = self.val_set().clone();
        if current_view.round > subject.view.round && subject.view.round > 0 {
            debug!("round change, current_round:{}, round:{}", current_view.round, subject.view.round, );
            // may be peer is less than network node
            self.send_round_change(subject.view.round);
            return Ok(());
        }

        let n = self
            .round_change_set
            .add(subject.view.round, msg.clone())
            .map_err(|err| ConsensusError::Unknown(err))?;
        debug!("round change, current_round:{}, round:{}, votes size {}", current_view.round, subject.view.round, n);

        // check round change more detail
//        if n >= (current_val_set.two_thirds_majority() + 1)
//            && (self.wait_round_change && current_view.round < subject.view.round) {
        if n >= (current_val_set.two_thirds_majority() + 1)
            && (current_view.round < subject.view.round) {
            // 注意：假设节点刚起动，这时候，其wait_round_change 可能未false，这样即使收到了超过+2/3的票，如果采用
            //  n == (current_val_set.two_thirds_majority() + 1, 是有问题的
            // receive more than local round and +2/3 has vote it
            self.send_round_change(subject.view.round);
            self.start_new_round(subject.view.round, &vec![]);
            return Ok(());
        } else if self.wait_round_change && current_view.round < subject.view.round {
            // receive more than local round
            return Err(ConsensusError::FutureRoundMessage);
        }
        Ok(())
    }
}

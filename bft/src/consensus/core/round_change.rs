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
        self.send_round_change(current_view.round + 1);
    }

    fn send_round_change(&mut self, round: Round) {
        if Instant::now().duration_since(self.round_change_limiter) <= Duration::from_millis(self.config.request_time) {
            debug!("Skip round change sent");
            self.new_round_change_timer();
            return;
        }
        self.round_change_limiter = Instant::now();

        self.catchup_round(round);
        let current_view = self.current_view();
        let ok = current_view.round < round;
        assert!(ok);

        // TODO add pre max round change prove
        let subject = Subject {
            view: View::new(current_view.height, round),
            digest: EMPTY_HASH,
        };
        let mut msg = GossipMessage::new(MessageType::RoundChange, subject.into_bytes(), None);
        msg.create_time = chrono::Local::now().timestamp_millis() as u64;
        self.broadcast(&msg);
    }

    fn handle(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult {
        debug!("Handle round change message from {:?}, from me: {}", src.address(), self.address() == *src.address());
        let subject: Subject = Subject::from_bytes(Cow::from(msg.msg()));
        self.check_message(MessageType::RoundChange, &subject.view)?;
        let current_view = self.current_view();
        let current_val_set = self.val_set().clone();

        let n = self
            .round_change_set
            .add(subject.view.round, msg.clone())
            .map_err(|err| ConsensusError::Unknown(err))?;
        debug!("round change votes size {:?}", n);

        // check round change more detail
        if self.wait_round_change && n == current_val_set.fault() {
            // receive more than local round and F has vote it
            if current_view.round < subject.view.round {
                self.send_round_change(subject.view.round);
            }
            return Ok(());
        } else if n == current_val_set.two_thirds_majority() + 1
            && (self.wait_round_change && current_view.round < subject.view.round) {
            // receive more than local round and +2/3 has vote it
            self.start_new_round(subject.view.round, &vec![]);
            return Ok(());
        } else if self.wait_round_change && current_view.round < subject.view.round {
            // receive more than local round
            return Err(ConsensusError::FutureRoundMessage);
        }
        Ok(())
    }
}

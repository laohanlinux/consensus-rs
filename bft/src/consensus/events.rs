use std::any::{Any, TypeId};

use ::actix::prelude::*;

use crate::{
    protocol::GossipMessage,
    types::Height,
};
use super::{
    types::{Proposal, View},
    error::ConsensusResult,
};

#[derive(Debug, Message)]
pub enum OpCMD {
    stop,
    Ping,
}

#[derive(Debug)]
pub enum RequestEventType {
    Block,
    Msg,
}

pub struct RequestEvent {
    proposal: Proposal,
}

fn is_view<T: ?Sized + Any>(_s: &T) -> bool {
    TypeId::of::<View>() == TypeId::of::<T>()
}

#[derive(Debug, Message)]
pub struct NewHeaderEvent {
    pub proposal: Proposal,
}

#[derive(Debug)]
pub struct MessageEvent {
    pub payload: Vec<u8>,
}

impl Message for MessageEvent {
    type Result = ConsensusResult;
}

#[derive(Debug, Message)]
pub struct FinalCommittedEvent {}

#[derive(Debug, Message)]
pub struct TimerEvent {}

#[derive(Debug)]
pub struct BackLogEvent {
    pub msg: GossipMessage,
}

impl Message for BackLogEvent {
    type Result = ConsensusResult;
}

#[derive(Debug, Message)]
pub enum ConsensusEvent {
    NetWork(MessageEvent),
    FinalCommitted(FinalCommittedEvent),
    Timer(TimerEvent),
    BackLog(BackLogEvent),
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct testView {}

    impl ::std::fmt::Display for testView {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "")
        }
    }

    #[test]
    fn test_type_of() {
        let view = View { height: 10, round: 20 };
        assert!(is_view(&view));
        assert!(!is_view(&testView {}));
    }
}
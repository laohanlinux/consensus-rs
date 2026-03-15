use std::any::{Any, TypeId};

use crate::protocol::GossipMessage;
use super::types::{Proposal, View};

#[derive(Debug, Clone, Copy)]
pub enum OpCMD {
    Stop,
    Ping,
}

#[derive(Debug)]
pub enum RequestEventType {
    Block,
    Msg,
}

pub struct RequestEvent {
    #[allow(dead_code)]
    proposal: Proposal,
}

#[allow(dead_code)]
fn is_view<T: ?Sized + Any>(_s: &T) -> bool {
    TypeId::of::<View>() == TypeId::of::<T>()
}

#[derive(Debug, Clone)]
pub struct NewHeaderEvent {
    pub proposal: Proposal,
}

#[derive(Debug, Clone)]
pub struct MessageEvent {
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FinalCommittedEvent {}

#[derive(Debug, Clone)]
pub struct TimerEvent {}

#[derive(Debug, Clone)]
pub struct BackLogEvent {
    pub msg: GossipMessage,
}

#[derive(Debug, Clone)]
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

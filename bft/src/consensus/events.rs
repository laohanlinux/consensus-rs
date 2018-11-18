use std::borrow::Cow;
use std::any::{Any, TypeId};

use super::types::{Proposal, View};
use crate::types::Height;

#[derive(Debug)]
pub enum RequestEventType{
    Block,
    Msg,
}

pub struct RequestEvent{
    proposal: Proposal,
}

fn is_view<T: ?Sized + Any>(_s: &T) -> bool {
    TypeId::of::<View>() == TypeId::of::<T>()
}

#[derive(Debug)]
pub struct MessageEvent {
    payload: Vec<u8>,
}


#[derive(Debug)]
pub struct FinalCommittedEvent{}

#[derive(Debug, Message)]
pub struct TimerEvent{}

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

//    impl Proposal for testView {
//        fn height(&self) -> Height {9}
//    }

    #[test]
    fn test_type_of(){
        let view = View{height:10, round:20};
        assert!(is_view(&view));
        assert!(!is_view(&testView{}));
    }
}
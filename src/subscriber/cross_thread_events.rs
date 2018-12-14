use ::actix::prelude::*;

#[derive(Message, Clone, Debug)]
pub enum ChainEvent {
    NewBlock,
    NewHeader,
}

pub mod E {
    use super::*;
    use crate::subscriber::impl_subscribe_handler;

    impl_subscribe_handler!{ChainEvent}
}
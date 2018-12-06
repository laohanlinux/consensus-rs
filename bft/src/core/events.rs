use crate::types::block::{Header, Block};

#[derive(Message, Clone, Debug)]
pub enum ChainEvent {
    NewBlock(Block),
    NewHeader(Header),
}
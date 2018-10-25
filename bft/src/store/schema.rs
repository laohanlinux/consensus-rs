use types::{Height, Bloom};
use types::block::{Block, Header};
use types::transaction::{Transaction};

macro_rules! define_name {
    (
        $(
            $name:ident => $value:expr;
        )+
    ) => (
        $(const $name: &str = concat!("core.", $value);)*
    );
}

define_name!(
    TRNSACTIONS => "transaction";
    BLOCKS => "blocks";
    BLOCK_HASHES_BY_HEIGHT => "block_hashes_by_height";
    BLOCK_TRANSACTIONS => "block_transactions";
    PRECOMMITS => "precommits";
    CONFIGS => "configs";
    CONSENSUS_MESSAGE_CACHE => "consensus_message_cache";
    CONSENSUS_ROUND => "consensus_round";
);

struct TxLocation {
    block_height: Height,
    position_in_block: u64,
}

pub struct Schema {
    prefix: Box<u8>,
}
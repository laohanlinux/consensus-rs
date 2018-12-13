use failure::Error;

use cryptocurrency_kit::crypto::Hash;

#[derive(Debug, Fail)]
pub enum TxPoolError {
    #[fail(display = "More than max txpool limit, max:{}", _0)]
    MoreThanMaxSIZE(u64),
}

#[derive(Debug, Fail)]
pub enum P2PError {
    #[fail(display = "Handshake fail")]
    HandShakeFailed,
    #[fail(display = "different genesis")]
    DifferentGenesis,
    #[fail(display = "Dump connected")]
    DumpConnected,
    #[fail(display = "Invalid Message type")]
    InvalidMessage,
    #[fail(display = "Timeout")]
    Timeout,
}

pub type ChainResult = Result<(), ChainError>;

#[derive(Debug, Fail)]
pub enum ChainError {
    #[fail(display = "the block has exist, ({:?})", _0)]
    Exists(Hash),
    #[fail(display = "An unknown error has occurred, ({})", _0)]
    Unknown(String),
}
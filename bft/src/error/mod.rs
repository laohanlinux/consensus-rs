use failure::Error;

#[derive(Debug, Fail)]
pub enum TxPoolError {
    #[fail(display = "More than max txpool limit, max:{}", _0)]
    MoreThanMaxSIZE(u64),
}

#[derive(Debug, Fail)]
pub enum P2PError {
    #[fail(display = "Handshake fail")]
    HandShakeFailed,
    #[fail(display = "Dump connected")]
    DumpConnected,
    #[fail(display = "Invalid Message type")]
    InvalidMessage,
}

pub type ChainResult = Result<(), ChainError>;

#[derive(Debug, Fail)]
pub enum ChainError{
    #[fail(display = "An unknown error has occurred, ({})", _0)]
    Unknown(String),
}
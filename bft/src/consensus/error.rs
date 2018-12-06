use failure::Error;

pub type ConsensusResult = Result<(), ConsensusError>;
pub type EngineResult = Result<(), EngineError>;

#[derive(Debug, Fail)]
pub enum ConsensusError {
    #[fail(display = "Ignore message")]
    Ignored,
    #[fail(display = "Future message")]
    FutureMessage,
    #[fail(display = "Future round message")]
    FutureRoundMessage,
    #[fail(display = "Future unit message")]
    FutureBlockMessage,
    #[fail(display = "inconsistent subjects")]
    InconsistentSubject,
    #[fail(display = "Old message")]
    OldMessage,
    #[fail(display = "Invalid message")]
    InvalidMessage,
    #[fail(display = "Unauthorized address")]
    UnauthorizedAddress,
    #[fail(display = "Waiting for new round")]
    WaitNewRound,
    #[fail(display = "Not from proposer")]
    NotFromProposer,
    #[fail(display = "Timeout message")]
    TimeoutMessage,
    #[fail(display = "An unknown error has occurred, ({})", _0)]
    Unknown(String),
    #[fail(display = "engine error hash occurred, ({})", _0)]
    Engine(EngineError)
}

#[derive(Debug, Fail)]
pub enum EngineError {
    #[fail(display = "engine is not started")]
    EngineNotStarted,
    #[fail(display = "Invalid proposal")]
    InvalidProposal,
    #[fail(display = "Invalid signature")]
    InvalidSignature,
    #[fail(display = "Invalid height")]
    InvalidHeight,
    #[fail(display = "Invalid timestamp")]
    InvalidTimestamp,
    #[fail(display = "Invalid transaction hash")]
    InvalidTransactionHash,
    #[fail(display = "Unauthorized")]
    Unauthorized,
    #[fail(display = "Lack votes, expect: {}, got: {}", _0, _1)]
    LackVotes(usize, usize),
    #[fail(display = "Block in the future")]
    FutureBlock,
    #[fail(display = "Invalid block number")]
    InvalidBlock,
    #[fail(display = "Unknown ancestor")]
    UnknownAncestor,
    #[fail(display = "Consensus interrupt")]
    Interrupt,
    #[fail(display = "An unknown error has occurred, ({})", _0)]
    Unknown(String),
}

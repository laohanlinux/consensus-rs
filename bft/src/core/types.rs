#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum State {
    AcceptRequest = 1,
    Preprepared,
    Prepared,
    Committed,
}
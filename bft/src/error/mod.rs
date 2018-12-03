use failure::Error;

#[derive(Debug, Fail)]
pub enum TxPoolError {
    #[fail(display = "More than max txpool limit, max:{}", _0)]
    MoreThanMaxSIZE(u64),
}
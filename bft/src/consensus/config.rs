#[derive(Debug, Clone)]
pub struct Config {
    pub request_time: u64,
    pub block_period: u64,
    pub chain_id: u64,
}

impl Config {
    pub fn new(request_time:u64, block_period: u64, chain_id: u64) -> Self {
        Config{
            request_time,
            block_period,
            chain_id,
        }
    }
}
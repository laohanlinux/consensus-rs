#[derive(Debug, Clone)]
struct Config {
    request_time: u64,
    block_period: u64,
}

impl Config {
    pub fn new(request_time:u64, block_period: u64) -> Self {
        Config{
            request_time,
            block_period,
        }
    }
}
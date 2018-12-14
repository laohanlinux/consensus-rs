#[derive(Default)]
pub struct Config {
    pub max_inbound: u64,
    pub max_outbound: u64,
    pub max_connection_size: u64,
    pub seal: bool,

}

impl Config {
//    pub fn new(max_inbound: u64, max_outbound: u64, max_connection_size: u64) -> Self {
//        Config {
//            max_inbound,
//            max_outbound,
//            max_connection_size,
//        }
//    }
}
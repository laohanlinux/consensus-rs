// Node - simplified, no longer an actix Actor
// Combines TcpServer and DiscoverService

use super::server::TcpServer;

pub struct Node {
    pub server: TcpServer,
}

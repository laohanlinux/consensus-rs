use actix::prelude::*;

use super::{
    server::{TcpServer, ServerEvent},
    discover_service::DiscoverService,
};

pub struct Node {
    server: Addr<TcpServer>,
    discover_service: Addr<DiscoverService>,
}


impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Node actor has started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Node actor has stopped");
    }
}
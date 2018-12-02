use actix::prelude::*;

use super::{
    server::{Server, ServerEvent},
    discover_service::DiscoverService,
};

pub struct Node {
    server: Addr<Server>,
    discover_service: Addr<DiscoverService>,
}


impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Node actor has started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("Node actor has stopped");
    }
}
use actix::prelude::*;

pub struct DiscoverService {
    pid: Addr<DiscoverService>,
}


impl Actor for DiscoverService {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        self.pid = ctx.address();
    }
}
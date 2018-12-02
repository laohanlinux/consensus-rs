use actix::prelude::*;

use std::time::Duration;

use crate::{
    consensus::validator::{ValidatorSet, ImplValidatorSet},
    consensus::events::TimerEvent,
};

use super::core::Core;

#[derive(Debug, Message)]
pub enum Op {
    Stop,
    Interval,
}

pub struct Timer{
    name: String,
    pub interval: Duration,
    pub pid: Addr<Core>,
}

impl Actor for Timer{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("{}'s timer actor has started", self.name);
        ctx.notify_later(Op::Interval, self.interval);
    }
}

impl Handler<Op> for Timer {
    type Result = ();
    fn handle(&mut self, msg: Op, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Op::Stop => ctx.stop(),
            Op::Interval => self.pid.do_send(TimerEvent {}),
        }
        ()
    }
}

impl Timer{
    pub fn new(name: String, interval: Duration, pid: Addr<Core>) -> Self {
        Timer { name, interval, pid }
    }
}
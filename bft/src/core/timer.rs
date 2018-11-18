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

pub struct Timer<T> where T: ValidatorSet + 'static {
    name: String,
    pub interval: Duration,
    pub pid: Addr<Core<T>>,
}

impl<T> Actor for Timer<T> where T: ValidatorSet + 'static{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("{}'s timer actor has started", self.name);
        ctx.notify_later(Op::Interval, self.interval);
    }
}

impl<T> Handler<Op> for Timer<T> where T: ValidatorSet + 'static {
    type Result = ();
    fn handle(&mut self, msg: Op, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Op::Stop => ctx.stop(),
            Op::Interval => self.pid.do_send(TimerEvent {}),
        }
        ()
    }
}

impl<T> Timer<T> where T: ValidatorSet {
    pub fn new(name: String, interval: Duration, pid: Addr<Core<T>>) -> Self {
        Timer { name, interval, pid }
    }
}
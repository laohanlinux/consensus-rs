use actix::prelude::*;

use std::time::Duration;


#[derive(Message)]
pub enum TimerOp {
    Stop
}

pub struct TimerRuntime {
    pub interval: Duration,
}

impl Actor for TimerRuntime {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(self.interval, |act, _| {
            System::current().stop();
        });
    }
}

impl Handler<TimerOp> for TimerRuntime {
    type Result = ();
    fn handle(&mut self, msg: TimerOp, _: &mut Context<Self>) -> Self::Result {
        System::current().stop();
    }
}
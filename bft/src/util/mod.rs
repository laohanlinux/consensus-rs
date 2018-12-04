use actix::prelude::*;

use std::time::Duration;


#[derive(Message)]
pub enum TimerOp {
    Stop
}

pub struct TimerRuntime {
    pub timeout: Duration,
}

impl Actor for TimerRuntime {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(self.timeout, |act, _| {
            System::current().stop();
            ::std::process::exit(0);
        });
    }
}

impl Handler<TimerOp> for TimerRuntime {
    type Result = ();
    fn handle(&mut self, msg: TimerOp, _: &mut Context<Self>) -> Self::Result {
        System::current().stop();
    }
}

impl TimerRuntime {
    pub fn new(timeout: Duration) -> Addr<TimerRuntime> {
        TimerRuntime::create(move |ctx| {
            TimerRuntime{
                timeout: timeout,
            }
        })
    }
}
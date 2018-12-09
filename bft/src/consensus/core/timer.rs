use ::actix::prelude::*;
use uuid::Uuid;

use std::time::Duration;

use crate::{
    consensus::validator::{ValidatorSet, ImplValidatorSet},
    consensus::events::{TimerEvent, BackLogEvent},
    protocol::GossipMessage,
    common::random_uuid,
};

use super::core::Core;

#[derive(Debug, Message)]
pub enum Op {
    Stop,
    Interval,
}

pub struct Timer {
    uuid: Uuid,
    name: String,
    pub interval: Duration,
    pub pid: Option<Addr<Core>>,
    msg: Option<GossipMessage>,
}

impl Actor for Timer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("[{:?}]{}'s timer actor has started, du:{:?}", self.uuid.to_string(), self.name, self.interval.as_secs());
        ctx.notify_later(Op::Interval, self.interval);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("[{:?}]{}'s timer actor has stopped", self.uuid.to_string(), self.name);
    }
}

impl Handler<Op> for Timer {
    type Result = ();
    fn handle(&mut self, msg: Op, ctx: &mut Self::Context) -> Self::Result {
        info!("[{:?}]{}'s timer actor triggers, op:{:?}", self.uuid.to_string(), self.name, msg);
        match msg {
            Op::Stop => ctx.stop(),
            Op::Interval => {
                if self.pid.is_some() {
                    if let Some(ref msg) = self.msg {
                        self.pid.as_ref().unwrap().do_send(BackLogEvent { msg: msg.clone() })
                    } else {
                        //FIXME
                        if self.name == "future" {
                            return;
                        }
                        self.pid.as_ref().unwrap().do_send(TimerEvent {})
                    };
                }
            }
        }
        ()
    }
}

impl Timer {
    pub fn new(name: String, interval: Duration, pid: Addr<Core>, msg: Option<GossipMessage>) -> Self {
        Timer { uuid: random_uuid(), name, interval, pid: Some(pid), msg: msg }
    }

    pub fn new_tmp(name: String, interval: Duration) -> Self {
        Timer { uuid: random_uuid(), name, interval, pid: None, msg: None }
    }
}
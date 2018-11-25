use actix::{Actor, Addr, Arbiter, Context, Handler, msgs, System};
use actix::AsyncContext;
use priority_queue::PriorityQueue;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::storage::values::StorageValue;
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use std::borrow::Cow;
use std::io::Cursor;
use std::collections::HashMap;
use std::time::Duration;

use crate::protocol::{GossipMessage, MessageType, to_priority};
use crate::consensus::types::{View, Subject, PrePrepare};
use crate::consensus::validator::ImplValidatorSet;
use super::core::Core;

pub struct BackLogActor {
    qp: HashMap<Address, PriorityQueue<GossipMessage, i64>>,
    core: Addr<Core>,
}


impl Actor for BackLogActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Back Log actor has started");
        self.process_back_log(ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("Back Log actor has stoppped");
    }
}

impl Handler<GossipMessage> for BackLogActor {
    type Result = ();
    fn handle(&mut self, msg: GossipMessage, ctx: &mut Context<Self>) -> Self::Result {
        match &msg.code {
            MessageType::Preprepare => {
                let msg_payload = msg.msg();
                let preprepare: PrePrepare = PrePrepare::from_bytes(Cow::from(msg_payload));
                let view = preprepare.view;
                let weight = to_priority(MessageType::Preprepare, view);
                self.qp.entry(msg.address).or_insert_with(||{
                    let mut qp = PriorityQueue::new();
                    qp.push(msg, weight);
                    qp
                });
            },
            other_code => {
                let msg_payload = msg.msg();
                let subject: Subject = Subject::from_bytes(Cow::from(msg_payload));
                let weight = to_priority(other_code.clone(), subject.view);
                self.qp.entry(msg.address).or_insert_with(||{
                    let mut qp = PriorityQueue::new();
                    qp.push(msg, weight);
                    qp
                });
            },
        }
        ()
    }
}

impl BackLogActor {
    fn process_back_log(&self, ctx: &mut actix::Context<Self>) {
        ctx.run_interval(Duration::from_millis(100), |act, ctx|{
            for (key, value) in act.qp.iter_mut(){
                for (message, _) in value.iter_mut() {
                    let mut view;
                    match &message.code {
                        MessageType::RoundChange => {
                            let preprepare: PrePrepare = PrePrepare::from_bytes(Cow::from(message.msg()));
                            view = preprepare.view;
                        },
                        other_type => {
                            let subject: Subject = Subject::from_bytes(Cow::from(message.msg()));
                            view = subject.view;
                        },
                    }

                }
            }
        });
    }
}


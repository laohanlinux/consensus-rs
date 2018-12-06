use actix::prelude::*;
use actix_broker::BrokerIssue;

use crate::types::block::{Header, Block};

#[derive(Message, Clone, Debug)]
pub enum ChainEvent {
    NewBlock(Block),
    NewHeader(Header),
}

pub enum SubscriberType {
    Async,
    Sync,
}

pub struct ChainEventSubscriber {
    subscriber_type: SubscriberType,
}

impl Actor for ChainEventSubscriber {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Chain event subscriber has started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("Chain event subscriber has stopped");
    }
}

impl Handler<ChainEvent> for ChainEventSubscriber {
    type Result = ();

    fn handle(&mut self, msg: ChainEvent, ctx: &mut Self::Context) -> Self::Result {
        match self.subscriber_type {
            SubscriberType::Async => {
                self.issue_async(msg);
            }
            SubscriberType::Sync => {
                self.issue_sync(msg, ctx);
            }
        }
    }
}

impl ChainEventSubscriber {
    pub fn new(subscriber_type: SubscriberType) -> Self {
        ChainEventSubscriber {
            subscriber_type: subscriber_type,
        }
    }
}


use crate::types::transaction::Transaction;
use crate::protocol::GossipMessage;

#[derive(Message, Clone, Debug)]
pub enum BroadcastEvent {
    Transaction(Transaction),
    Block(Block),
    Consensus(GossipMessage),
}

pub struct BroadcastEventSubscriber {
    subscriber_type: SubscriberType,
}

impl Actor for BroadcastEventSubscriber {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Broadcast event subscriber has started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("Broadcast event subscriber has stopped");
    }
}

impl Handler<BroadcastEvent> for BroadcastEventSubscriber {
    type Result = ();

    fn handle(&mut self, msg: BroadcastEvent, ctx: &mut Self::Context) -> Self::Result {
        match self.subscriber_type {
            SubscriberType::Async => {
                self.issue_async(msg);
            }
            SubscriberType::Sync => {
                self.issue_sync(msg, ctx);
            }
        }
    }
}

impl BroadcastEventSubscriber {
    pub fn new(subscriber_type: SubscriberType) -> Self {
        BroadcastEventSubscriber {
            subscriber_type: subscriber_type,
        }
    }
}
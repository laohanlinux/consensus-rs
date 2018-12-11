use ::actix::prelude::*;
use actix_broker::BrokerIssue;

use crate::types::block::{Header, Block};

#[derive(Message, Clone, Debug)]
pub enum ChainEvent {
    NewBlock(Block),
    NewHeader(Header),
}

// cross thread event
pub mod ChainEventCT {
    use ::actix::prelude::*;
    use super::ChainEvent;
    use crate::subscriber::impl_subscribe_handler;

    impl_subscribe_handler!{ChainEvent}
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

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Chain event subscriber has started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
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

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Broadcast event subscriber has started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Broadcast event subscriber has stopped");
    }
}

impl Handler<BroadcastEvent> for BroadcastEventSubscriber {
    type Result = ();

    fn handle(&mut self, msg: BroadcastEvent, ctx: &mut Self::Context) -> Self::Result {
        debug!("BroadcastEventSubscriber[e:BroadcastEvent]");
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

#[cfg(test)]
mod test {
    use super::*;
    use actix_broker::BrokerSubscribe;
    use actix_broker::Broker;

    struct ProActor {
        name: String,
    }

    impl Actor for ProActor {
        type Context = Context<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {
            self.subscribe_async::<BroadcastEvent>(ctx);
        }
    }

    impl Handler<BroadcastEvent> for ProActor {
        type Result = ();

        fn handle(&mut self, msg: BroadcastEvent, _ctx: &mut Self::Context) {
            println!("ProActor[{}] Received: {:?}", self.name, msg);
        }
    }

    struct SubActor {}

    impl Actor for SubActor {
        type Context = Context<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {}
    }

    impl Handler<BroadcastEvent> for SubActor {
        type Result = ();

        fn handle(&mut self, msg: BroadcastEvent, _ctx: &mut Self::Context) {
            println!("SubActor Received: {:?}", msg);
           // self.issue_async(msg);
            Broker::issue_async(msg);
        }
    }

    #[test]
    fn t_async_actor() {
        use crate::protocol::{GossipMessage, MessageType};
        crate::logger::init_test_env_log();
        let pro = ProActor { name: "Same thread".to_owned() }.start();

        ::std::thread::spawn(move || {
            System::run(move || {
                let pro = ProActor { name: "Cross thread".to_owned() }.start();
            });
        });

        // customer
//        let sub = BroadcastEventSubscriber { subscriber_type: SubscriberType::Async }.start();
        let sub = SubActor {}.start();

        ::std::thread::spawn(move || {
            while true {
                sub.do_send(BroadcastEvent::Consensus(GossipMessage::new(MessageType::RoundChange, vec![], None)));
                ::std::thread::sleep(::std::time::Duration::from_secs(2));
            }
        });

        crate::pprof::spawn_signal_handler(*crate::common::random_dir());
    }
}
#[macro_use]
use ::actix::prelude::*;
use libp2p::Multiaddr;
use libp2p::PeerId;

pub mod async_subscriber;
pub mod events;
pub mod cross_thread_events;

use crate::types::block::{Block, Header};
use super::*;

#[derive(Message, Clone, Debug)]
pub enum P2PEvent {
    AddPeer(PeerId, Vec<Multiaddr>),
    DropPeer(PeerId, Vec<Multiaddr>),
}

impl_subscribe_handler! {P2PEvent}

pub fn spawn_sync_subscriber() -> Addr<ProcessSignals> {
    Actor::create(|_| ProcessSignals {
        subscribers: vec![],
    })
}

#[macro_export]
macro_rules! impl_subscribe_handler {
    ($key: ident) => {
        #[derive(Message)]
        pub enum SubscribeMessage {
            SubScribe(Recipient<$key>),
            UnSubScribe(Recipient<$key>),
        }

        impl SubscribeMessage {
            pub fn new_subScribe(recipient: Recipient<$key>) -> Self {
                SubscribeMessage::SubScribe(recipient)
            }

            pub fn new_unsubScribe(recipient: Recipient<$key>) -> Self {
                SubscribeMessage::UnSubScribe(recipient)
            }
        }

        #[derive(Clone)]
        pub struct ProcessSignals {
            subscribers: Vec<Recipient<$key>>,
        }

        impl Actor for ProcessSignals {
            type Context = Context<Self>;
        }

        impl Handler<$key> for ProcessSignals {
            type Result = ();
            fn handle(&mut self, msg: $key, _: &mut Self::Context) {
                trace!("Receive a notify message");
                self.distribute(msg);
            }
        }

        impl Handler<SubscribeMessage> for ProcessSignals {
            type Result = ();

            fn handle(&mut self, msg: SubscribeMessage, _: &mut Self::Context) {
                trace!("Receive a subscibe message");
                match msg {
                    SubscribeMessage::SubScribe(recipient) => {
                        self.subscribe(recipient);
                    }
                    SubscribeMessage::UnSubScribe(recipient) => {
                        self.unsubscribe(recipient);
                    }
                }
            }
        }

        impl ProcessSignals {
            pub fn new() -> Self {
                ProcessSignals {subscribers: vec![]}
            }

            pub fn subscribe(&mut self, recipient: Recipient<$key>) {
                self.subscribers.push(recipient);
            }

            pub fn unsubscribe(&mut self, recipient: Recipient<$key>) {
                self.subscribers.remove_item(&recipient);
            }

            /// Async send a message to subscriber mailbox
            pub fn distribute(&mut self, msg: $key) {
                for subscriber in &self.subscribers {
                    subscriber.do_send(msg.clone());
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use futures::future;
    use futures::Future;
    use super::*;
    use std::io::{self, Write};

    struct Worker {
        name: String,
    }

    impl Actor for Worker {
        type Context = Context<Self>;
    }

    #[derive(Message, Debug, Clone)]
    pub struct RawMessage {
        tm: std::time::Duration,
    }

    impl Handler<RawMessage> for Worker {
        type Result = ();
        fn handle(&mut self, msg: RawMessage, _: &mut Self::Context) {
            use std::io::{self, Write};
            writeln!(io::stdout(), "[{}] worker receive a msg: {:?}", self.name, msg).unwrap();
        }
    }

    impl_subscribe_handler! {RawMessage}

    #[test]
    fn t_subscribe() {
        use chrono::Local;
        use chrono::Timelike;
        let system = System::new("test");
        let subscribe_pid = Actor::create(|_| ProcessSignals {
            subscribers: vec![],
        });
        (0..10).for_each(|_idx| {
            let name = format!("{}", Local::now().time().nanosecond());
            let worker = Worker::create(|_| Worker {
                name: name,
            });
            let recipient = worker.recipient();
            let message = SubscribeMessage::SubScribe(recipient);
            let request = subscribe_pid.clone().send(message);
            Arbiter::spawn(request.then(|_| {
                future::result(Ok(()))
            }));
        });

        (0..230).for_each(|idx| {
            subscribe_pid.do_send(RawMessage {
                tm: std::time::Duration::from_secs(idx),
            });
        });
        Arbiter::spawn_fn(|| {
            System::current().stop();
            future::ok(())
        });
        system.run();
    }
}

#[macro_use]
use actix::prelude::*;
use libp2p::PeerId;
use libp2p::Multiaddr;

#[macro_use]
use super::*;


#[derive(Message, Clone, Debug)]
pub enum P2PEvent {
    AddPeer(PeerId, Vec<Multiaddr>),
    DropPeer(PeerId, Vec<Multiaddr>),
}

impl_subscribe_handler! {P2PEvent}

pub fn spawn_sync_subscriber() -> Addr<ProcessSignals> {
    Actor::create(|_| {
        ProcessSignals{
            subscribers: vec![],
        }
    })
}

#[macro_export]
macro_rules! impl_subscribe_handler {
    ($key: ident) => {
        #[derive(Message)]
        pub struct Subscribe(pub Recipient<$key>);

        #[derive(Message)]
        pub enum SubscribeMessage {
            SubScribe(Recipient<$key>),
            UnSubScribe(Recipient<$key>),
        }

        #[derive(Clone)]
        pub struct ProcessSignals {
            subscribers: Vec<Recipient<$key>>,
        }

        impl Actor for ProcessSignals {
            type Context = Context<Self>;
        }

        impl Handler<Subscribe> for ProcessSignals {
            type Result = ();

            fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) {
                self.subscribers.push(msg.0);
            }
        }

        impl Handler<$key> for ProcessSignals {
            type Result = ();
            fn handle(&mut self, msg: $key, ctx: &mut Self::Context) {
                self.distribute(msg);
            }
        }

        impl Handler<SubscribeMessage> for ProcessSignals {
            type Result = ();

            fn handle(&mut self, msg: SubscribeMessage, ctx: &mut Self::Context) {
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
            pub fn subscribe(&mut self, recipient: Recipient<$key>) {
                self.subscribers.push(recipient);
            }

            pub fn unsubscribe(&mut self, recipient: Recipient<$key>) {
                self.subscribers.remove_item(&recipient);
            }

            pub fn distribute(&mut self, msg: $key) {
                for subscriber in &self.subscribers {
                    subscriber.do_send(msg.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    struct Worker {}

    impl Actor for Worker {
        type Context = Context<Self>;
    }

    #[derive(Message, Debug, Clone)]
    pub struct RawMessage {}

    impl Handler<RawMessage> for Worker {
        type Result = ();
        fn handle(&mut self, msg: RawMessage, _: &mut Self::Context) {
            use std::io::{self, Write};
            writeln!(io::stdout(), "work receive a msg: {:?}", msg);
        }
    }

    impl_subscribe_handler! {RawMessage}

    #[test]
    fn t_subscribe() {
        let system = System::new("test");
        let subscribe_pid = Actor::create(|_| {
            ProcessSignals {
                subscribers: vec![],
            }
        });

        let worker = Worker::create(|_| {
            Worker {}
        });
        let recipient = worker.recipient();
        let message = SubscribeMessage::SubScribe(recipient);
        let request = subscribe_pid.send(message);
        writeln!(io::stdout(), "get request");
        let request = subscribe_pid.send(RawMessage {});
        writeln!(io::stdout(), "get request");
        system.run();
    }
}



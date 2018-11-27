use actix::prelude::*;

#[derive(Message)]
pub struct Signal(usize);

#[derive(Message)]
struct Subscribe(pub Recipient<Signal>);

#[derive(Message)]
pub enum SubscribeMessage<T> {
    SubScribe(T),
    UnSubScribe(T),
    RawMessage(T),
}

#[derive(Clone)]
pub struct ProcessSignals {
    pub pid: Option<Addr<ProcessSignals>>,
    subscribers: Vec<Recipient<Signal>>,
}

impl Actor for ProcessSignals {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Context<Self>) {
        // add stream
    }
}

impl ProcessSignals {
    fn send_signal(&mut self, sig: usize) {
        for subscribe in &self.subscribers {
            subscribe.do_send(Signal(sig));
        }
    }

    pub fn subscribe(&mut self) {
        use std::io::{self, Write};
        writeln!(io::stdout(), "receive a subscribe message");
    }
    pub fn unsubscribe(&mut self) {}

    pub fn distribute(&mut self) {}
}

impl Handler<Subscribe> for ProcessSignals {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) {
        self.subscribers.push(msg.0);
    }
}

impl<T> Handler<SubscribeMessage<T>> for ProcessSignals {
    type Result = ();

    fn handle(&mut self, msg: SubscribeMessage<T>, ctx: &mut Self::Context) {
        use std::io::{self, Write};
        writeln!(io::stdout(), "DoDoDoDoDo");
        match msg {
            SubscribeMessage::SubScribe(ref recipient) => {
                self.subscribe();
            }
            SubscribeMessage::UnSubScribe(ref recipient) => {
                self.unsubscribe();
            }
            SubscribeMessage::RawMessage(_) => {}
        }
//        ctx.address().send()
        ()
    }
}

struct Worker {}

impl Actor for Worker {
    type Context = Context<Self>;
}

impl Handler<Signal> for Worker {
    type Result = ();
    fn handle(&mut self, msg: Signal, _: &mut Self::Context) {
        use std::io::{self, Write};
        writeln!(io::stdout(), "receive a signal: {}", msg.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    #[test]
    fn t_subscribe() {
        let system = System::new("test");
        let subscribe_pid = Actor::create(|_| {
            ProcessSignals {
                pid: None,
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
        ::std::thread::spawn(|| {});
        ::std::thread::spawn(|| {
            ::std::thread::sleep(
                ::std::time::Duration::from_secs(1)
            );
        });

        system.run();
//        addr.send(Subscribe(worker.recipient::<Signal>()));
//
    }
}

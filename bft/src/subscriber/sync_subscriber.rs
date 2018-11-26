use actix::prelude::*;

#[derive(Message)]
struct Signal(usize);

#[derive(Message)]
struct Subscribe(pub Recipient<Signal>);

pub struct ProcessSignals{
    subscribers: Vec<Recipient<Signal>>,
}

impl Actor for ProcessSignals {
    type Context = Context<Self>;
}

impl ProcessSignals {
    fn send_signal(&mut self, sig: usize) {
        for subscr in &self.subscribers {
            subscr.do_send(Signal(sig));
        }
    }
}

impl Handler<Subscribe> for ProcessSignals {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _:&mut Self::Context) {
        self.subscribers.push(msg.0);
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn t_subscribe(){
        let system = System::new("test");
        let addr = ProcessSignals::create(|_| {
            ProcessSignals{
                subscribers: Vec::new(),
            }
        });

        system.run();
    }
}
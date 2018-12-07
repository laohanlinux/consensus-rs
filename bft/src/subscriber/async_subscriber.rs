use ::actix::prelude::*;


pub struct Subscribe<M: Message>(pub Recipient<M>)
    where M: Message + Send,
          M::Result: Send;

pub enum SubscribeMessage<M: Message>
    where M: Message + Send,
          M::Result: Send,
{
    SubScribe(Recipient<M>),
    UnSubScribe(Recipient<M>),
}

pub struct ProcessSignals<M: Message>
    where M: Message + Send,
          M::Result: Send,
{
    subscribers: Vec<Recipient<M>>,
}

//impl<M: Message> Handler<SubscribeMessage<M>> for ProcessSignals<M>
//    where M: Message + Send,
//          M::Result: Send,
//{
//    type Result = ();
//
//    fn handle(&mut self, msg: SubscribeMessage<M>, _: &mut Self::Context) {
//        match msg {
////            SubscribeMessage::SubScribe(recipient) => {
////                self.subscribe(recipient);
////            }
////            SubscribeMessage::UnSubScribe(recipient) => {
////                self.unsubscribe(recipient);
////            }
//        }
//    }
//}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Message, Clone)]
    pub struct Ping {}

    impl Message for Subscribe<Ping> {
        type Result = ();
    }

    #[test]
    fn tt() {}
}
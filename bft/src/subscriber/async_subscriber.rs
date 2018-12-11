pub mod E {
    use ::actix::prelude::*;
    use crate::subscriber::impl_subscribe_handler;

    #[derive(Message, Clone, Debug)]
    pub struct Event1 {}
    impl_subscribe_handler!{Event1}
}
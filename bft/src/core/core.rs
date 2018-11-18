use actix::prelude::*;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::ethkey::Signature;

use std::hash::Hash as StdHash;
use std::time::Duration;

use crate::{
    types::Validator,
    consensus::events::TimerEvent,
    consensus::types::{View, Subject, Proposal},
    consensus::config::Config,
    consensus::validator::{Validators, ValidatorSet, ImplValidatorSet},
    protocol::{GossipMessage as ProtoMessage, MessageType},
};
use super::{
    types::State,
    timer::{Timer, Op},
    round_state::RoundState,
};

pub struct Core<V: ValidatorSet + 'static> {
    pid: Addr<Core<V>>,
    config: Config,

    address: Address,
    state: State,

    validators: V,
    current_state: RoundState<Proposal>,

    future_prepprepare_timer: Addr<Timer<V>>,
    round_change_timer: Addr<Timer<V>>,
}

impl<V> Actor for Core<V>
    where V: ValidatorSet + 'static
{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("core actor has started");
        self.pid = ctx.address();
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("core actor has stopped");
    }
}

impl<V> Handler<ProtoMessage> for Core<V>
    where V: ValidatorSet + 'static
{
    type Result = ();

    fn handle(&mut self, msg: ProtoMessage, ctx: &mut Self::Context) -> Self::Result {
        ()
    }
}

impl<V> Handler<TimerEvent> for Core<V>
    where V: ValidatorSet + 'static
{
    type Result = ();

    fn handle(&mut self, msg: TimerEvent, ctx: &mut Self::Context) -> Self::Result {
        ()
    }
}


impl<V> Core<V>
    where V: ValidatorSet + 'static
{
    pub fn check_message(&self, code: MessageType, view: &View) -> Result<(), String> {
        if view.height == 0 {
            return Err("invalid view, height should be zero".to_string());
        }

        Ok(())
    }


    pub(crate) fn update_round_state(&mut self, view: View, vals: &V, round_change: bool) {
        debug!("update round state");
        // 如果已经锁定在某一个高度，则应该继承其锁，且下一轮次继续以锁定的提案进行`共识`
        if self.current_state.is_locked() {
//            self.current_state = RoundState::lock_hash()
        }
    }

    pub fn set_state(&mut self, new_state: State) {
        trace!("state change, from {:?} to {:?}", self.state, new_state);
        self.state = new_state;
    }

    pub fn address(&self) -> Address {
        self.address
    }

    fn stop_future_preprepare_timer(&mut self) {
        // stop old timer
        self.future_prepprepare_timer.try_send(Op::Stop);
        // start new timer
//        let pid = self.pid.clone();
//        self.future_prepprepare_timer = Timer::create(move|timer_ctx|{
//            Timer::new("future preprepare".to_string(), Duration::from_millis(3*1000), pid)
//        });
    }

    fn stop_round_change_timer(&mut self) {
        self.round_change_timer.try_send(Op::Stop);
        info!("Stop round change timer");
    }

    fn stop_timer(&mut self) {
        self.stop_future_preprepare_timer();
        self.stop_round_change_timer();
    }

    fn new_round_change_timer(&mut self) {
        // stop old timer
        self.round_change_timer.try_send(Op::Stop);
        // start new timer
        let pid = self.pid.clone();
        self.round_change_timer = Timer::create(move |timer_ctx| {
            Timer::new("round change".to_string(), Duration::from_millis(3 * 1000), pid)
        })
    }
}
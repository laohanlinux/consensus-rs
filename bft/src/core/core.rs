use actix::prelude::*;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::ethkey::Signature;
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use std::borrow::Borrow;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::io::Cursor;
use std::any::{Any, TypeId};
use std::hash::Hash as StdHash;
use std::time::Duration;

use crate::{
    types::Validator,
    consensus::events::TimerEvent,
    consensus::types::{Round, View, Request as CSRequest, Subject, Proposal},
    consensus::config::Config,
    consensus::backend::Backend,
    consensus::validator::{Validators, ValidatorSet, ImplValidatorSet},
    protocol::{GossipMessage as ProtoMessage, MessageType},
};
use super::{
    types::State,
    timer::{Timer, Op},
    round_state::RoundState,
    round_change_set::RoundChangeSet,
    preprepare::HandlePreprepare,
};

pub struct Core {
    pid: Addr<Core>,
    config: Config,

    address: Address,
    pub state: State,

    validators: ImplValidatorSet,
    current_state: RoundState,
    // 轮次的状态，存储本轮次的消息
    round_change_set: RoundChangeSet<ImplValidatorSet>, // store round change messages

    wait_round_change: bool,
    future_prepprepare_timer: Addr<Timer>,
    round_change_timer: Addr<Timer>,

    backend: Box<Backend<ValidatorsType=ImplValidatorSet>>,
}

impl Actor for Core
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

impl Handler<ProtoMessage> for Core
{
    type Result = ();

    fn handle(&mut self, msg: ProtoMessage, ctx: &mut Self::Context) -> Self::Result {
        ()
    }
}

impl Handler<TimerEvent> for Core
{
    type Result = ();

    fn handle(&mut self, msg: TimerEvent, ctx: &mut Self::Context) -> Self::Result {
        ()
    }
}


impl Core
{
    pub fn check_message(&self, code: MessageType, view: &View) -> Result<(), String> {
        if view.height == 0 {
            return Err("invalid view, height should be zero".to_string());
        }

        Ok(())
    }

    // enter commit state
    pub fn commit(&mut self) {
        self.set_state(State::Committed);
        let proposal = self.current_state.proposal().unwrap();
        let mut committed_seals = Vec::with_capacity(self.current_state.commits.len());
        self.current_state.commits.values().iter().for_each(|v| {
            committed_seals.push(v.signature.as_ref().unwrap().clone());
        });
        let has_more_than_maj23 = self.validators.two_thirds_majority() + 1 <= committed_seals.len();
        assert!(has_more_than_maj23);
        // TODO commit
    }



    // 启动新的轮次，触发的条件
    // 1：新高度初始化
    // 2：锁定+2/3的 round change 票
    pub(crate) fn start_new_zero_round(&mut self) {
        trace!("before start zero round");
        let last_proposal = self.backend.last_proposal().unwrap();
        let last_proposer = last_proposal.block().coinbase();
        let last_height = last_proposal.block().height();
        // TODO 增加判断，last_proposal == blockend.proposal_hash
        let mut new_view: View = View::new(last_height + 1, 0);
        // TODO 从backend 获取 backend.validator_set
        self.validators = self.backend.validators(last_height + 1).clone();
        self.round_change_set = RoundChangeSet::new(self.validators.clone(), None);
        assert_ne!(self.validators.size(), 0, "validators'size should be more than zero");

        // New snapshot for new round
        self.update_round_state(new_view, self.validators.clone(), false);
        // calc new proposer
        self.validators.calc_proposer(&last_proposal.block().hash(), last_height, new_view.round);

        // reset state
        self.wait_round_change = false;
        // set state into State::AcceptRequest
        // NOTIC: the next step should set request atomic
        self.set_state(State::AcceptRequest);
        // reset new round change timer
        self.new_round_change_timer();
        info!("after start zero round");
    }

    // has receive +2/3 round change
    pub(crate) fn start_new_round(&mut self, round: Round, pre_change_prove: &[u8]) {
        trace!("before start new round");
        assert_ne!(round, 0, "zero round only call by self.start_new_zero_round");
        let expect = round > self.current_state.round();
        assert!(expect, "new round should not be smaller than or equal current round");
        let last_proposal = self.backend.last_proposal().unwrap();
        let last_proposer = last_proposal.block().coinbase();
        let last_height = last_proposal.block().height();
        {
            let got = (last_height + 1) < self.current_state.height();
            assert_eq!(got, false);
        }

        if last_height > self.current_state.height() {
            // 本地的高度等于当前正在做共识的高度，证明网络上已经有新的高度了
            trace!("catchup latest proposal, it should be not happen");
            return;
        }
        // last_height + 1 = current_state.height

        trace!("ready to update round, because round change");
        let mut new_view = View::new(self.current_state.height(), self.current_state.round());

        assert_ne!(self.validators.size(), 0, "validators'size should be more than zero");

        // start new round timer
        let max_round = self.round_change_set.max_round(self.validators.two_thirds_majority() + 1);

        // TODO 继承上一次的Round change prove
        // round change
        // TODO prove tree
        self.round_change_set = RoundChangeSet::new(self.validators.clone(), None);


        // New snapshot for new round
        self.update_round_state(new_view, self.validators.clone(), true);
        // calc new proposer
        self.validators.calc_proposer(&last_proposal.block().hash(), last_height, new_view.round);

        // reset state
        self.wait_round_change = false;
        // set state into State::AcceptRequest
        // NOTIC: the next step should set request atomic
        self.set_state(State::AcceptRequest);

        // if current validator is proposer
        if self.validators.is_proposer(self.address) {
            // if it is locked, propose the old proposal, if we have pending request. propose pending request
            if self.current_state.is_locked() {
                // c.current_state.proposal has locked by previous proposer, see update_round_state
                let r = CSRequest::new(self.current_state.proposal().unwrap().clone());
                self.send_preprepare(&r);
                // TODO
            } else {
                // TODO
                self.send_preprepare(self.current_state.pending_request.as_ref().unwrap());
            }
        }

        // reset new round change timer
        self.new_round_change_timer();
        info!("after start new round, new round: {}", self.current_state.round());
    }

    // 处理新的round
    // 等待+2/3
    pub(crate) fn catchup_round(&mut self, view: &View) {
        trace!("catchup new round, current round:{}, new round: {}", self.current_state.round(), view.round);
        // 设置当前状态为wait for round change
        self.wait_round_change = true;
        // 启动新的时钟
        self.new_round_change_timer();
    }

    // TODO 修复不同节点锁定的提案不一致时，需要采用某种手段去修复
    // 如以锁定的周期最新为基点
    pub(crate) fn update_round_state(&mut self, view: View, vals: ImplValidatorSet, round_change: bool) {
        debug!("update round state");
        // 来自于轮次的改变
        if round_change {
            // 已经锁定在某一个高度，则应该继承其锁，且下一轮次继续以锁定的提案进行`共识`
            if self.current_state.is_locked() {
                self.current_state = RoundState::new_round_state(view, vals,
                                                                 self.current_state.get_lock_hash(),
                                                                 self.current_state.preprepare.clone(),
                                                                 self.current_state.pending_request.take());
            } else {
                // 未锁定到某个提案
                self.current_state = RoundState::new_round_state(view, vals, None, None, self.current_state.pending_request.take());
            }
        } else {
            // 来之新的高度，或者初始化的逻辑
            self.current_state = RoundState::new_round_state(view, vals, None, None, None);
        }
    }

    pub fn set_state(&mut self, new_state: State) {
        trace!("state change, from {:?} to {:?}", self.state, new_state);
        self.state = new_state;
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn mut_current_state(&mut self) -> &mut RoundState {
        &mut self.current_state
    }

    pub fn val_set(&self) -> &ImplValidatorSet{
        &self.validators
    }


    fn stop_future_preprepare_timer(&mut self) {
        // stop old timer
        self.future_prepprepare_timer.try_send(Op::Stop);
    }

    fn stop_round_change_timer(&mut self) {
        self.round_change_timer.try_send(Op::Stop);
        info!("stop round change timer");
    }

    fn stop_timer(&mut self) {
        self.stop_future_preprepare_timer();
        self.stop_round_change_timer();
    }

    fn new_round_change_timer(&mut self) {
        trace!("start new round timer");
        // stop old timer
        self.round_change_timer.try_send(Op::Stop);
        // start new timer
        let pid = self.pid.clone();
        self.round_change_timer = Timer::create(move |timer_ctx| {
            Timer::new("round change".to_string(), Duration::from_millis(3 * 1000), pid)
        })
    }
}
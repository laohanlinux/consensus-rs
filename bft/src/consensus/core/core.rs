use ::actix::prelude::*;
use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::{hash, CryptoHash, Hash};
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::ethkey::{KeyPair, Signature};
use rmps::decode::Error;
use rmps::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use futures::Future;

use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::io::Cursor;
use std::time::Duration;
use std::time::Instant;
use std::sync::Arc;

use super::{
    request::HandlerRequst,
    preprepare::HandlePreprepare,
    prepare::HandlePrepare,
    commit::HandleCommit,
    round_change::HandleRoundChange,
    round_change_set::RoundChangeSet,
    round_state::RoundState,
    timer::{Op, Timer},
    back_log::BackLogActor,
};
use crate::{
    core::chain::Chain,
    consensus::validator::fn_selector,
    consensus::backend::{Backend, ImplBackend},
    consensus::config::Config,
    consensus::error::{ConsensusError, ConsensusResult},
    consensus::events::{OpCMD, MessageEvent, NewHeaderEvent, FinalCommittedEvent, BackLogEvent, TimerEvent},
    consensus::types::{Proposal, Request as CSRequest, Round, Subject, View},
    consensus::validator::{ImplValidatorSet, ValidatorSet, Validators},
    p2p::server::HandleMsgFn,
    p2p::protocol::{RawMessage, P2PMsgCode, Payload},
    protocol::{GossipMessage, MessageType, State},
    types::Validator,
    types::block::Block,
};

pub fn handle_msg_middle(core_pid: Addr<Core>, chain: Arc<Chain>) -> impl Fn(RawMessage) -> Result<(), String> {
    move |msg: RawMessage| {
        let header = msg.header();
        let payload = msg.payload().to_vec();
        match header.code {
            P2PMsgCode::Consensus => {
                let request = core_pid.send(MessageEvent { payload: payload });
                Arbiter::spawn(request.and_then(|result| {
                    if let Err(err) = result {
                        error!("Failed to handle message, err:{:?}", err);
                    }
                    futures::future::ok(())
                }).map_err(|err| panic!(err)));
            }
            P2PMsgCode::Block => {
                let block = Block::from_bytes(Cow::from(&payload));
                info!("Receive a new block from network, hash: {:?}, height: {:?}", block.hash(), block.height());
                chain.insert_block(&block);
            }
            _ => unimplemented!()
        }

        Ok(())
    }
}


pub struct Core {
    pid: Addr<Core>,
    pub config: Config,

    address: Address,
    pub keypair: KeyPair,
    pub state: State,

    validators: ImplValidatorSet,
    pub current_state: RoundState,
    // 轮次的状态，存储本轮次的消息
    pub round_change_set: RoundChangeSet<ImplValidatorSet>, // store round change messages

    pub wait_round_change: bool,
    future_prepprepare_timer: Addr<Timer>,
    round_change_timer: Addr<Timer>,
    pub consensus_timestamp: Duration,

    backlog_store: Addr<BackLogActor>,
    pub backend: Box<Backend<ValidatorsType=ImplValidatorSet>>,
    pub round_change_limiter: Instant,
}

impl Actor for Core {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("core actor has started");
        self.start_new_zero_round();
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("core actor has stopped");
    }
}

impl Handler<NewHeaderEvent> for Core {
    type Result = ();

    fn handle(&mut self, msg: NewHeaderEvent, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Receive a new header event");
        let proposal = msg.proposal.clone();
        self.start_new_zero_round();
        let result: ConsensusResult = <Core as HandlerRequst>::handle(self, &CSRequest::new(proposal));
        assert_eq!(result.is_ok(), true);
        ()
    }
}

impl Handler<FinalCommittedEvent> for Core {
    type Result = ();

    fn handle(&mut self, _msg: FinalCommittedEvent, _ctx: &mut Self::Context) -> Self::Result {
        self.stop_timer();
        self.wait_round_change = false;
        ()
    }
}

impl Handler<MessageEvent> for Core {
    type Result = ConsensusResult;

    fn handle(&mut self, msg: MessageEvent, _ctx: &mut Self::Context) -> Self::Result {
        let result = self.handle_message(&msg.payload);
        if let Err(ref err) = result {
            error!("Failed to handle message, err: {:?}", err);
        }
        result
    }
}

impl Handler<BackLogEvent> for Core {
    type Result = ConsensusResult;

    fn handle(&mut self, msg: BackLogEvent, _ctx: &mut Self::Context) -> Self::Result {
        let msg = msg.msg;
        let src = Validator::new(msg.address);
        self.handle_check_message(&msg, &src)
    }
}

impl Handler<TimerEvent> for Core {
    type Result = ();

    fn handle(&mut self, _msg: TimerEvent, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Receive timer event");
        let last_proposal = self.backend.last_proposal().unwrap();
        let last_block = last_proposal.block();
        let cur_view = self.current_view();
        if last_block.height() >= cur_view.height {
            info!("Round change timeout, catch up latest height");
            self.stop_timer();
            self.wait_round_change = false;
        } else {
            // send new round message
            self.send_next_round_change();
        }
        ()
    }
}

impl Handler<OpCMD> for Core {
    type Result = ();

    fn handle(&mut self, msg: OpCMD, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            OpCMD::stop => {
                self.stop_timer();
                ctx.stop();
            }
            OpCMD::Ping => {
                debug!("Recive a test message");
            }
        }

        ()
    }
}

impl Core {
    pub fn new(chain: Arc<Chain>, backend: ImplBackend, key_pair: KeyPair) -> Addr<Core> {
        //    let core_backend: Box<Backend<ValidatorsType=ImplValidatorSet> + Send + Sync> = Box::new(backend.clone()) as Box<Backend<ValidatorsType=ImplValidatorSet> + Send + Sync>;
        let address = key_pair.address();
        let last_block = chain.get_last_block();
        let validators = chain.get_validators(last_block.height());
        let addresses: Vec<Address> = validators.iter().map(|validator| *validator.address()).collect();
        let validators = ImplValidatorSet::new(&addresses, Box::new(fn_selector));

        let last_view = View::new(last_block.height(), 0);
        let lock_hash = last_block.hash();
        let current_state = RoundState::new_round_state(last_view,
                                                        validators.clone(),
                                                        Some(lock_hash),
                                                        None,
                                                        None);
        let round_change_set = RoundChangeSet::new(validators.clone(), None);

        let request_time = Duration::from_millis(chain.config.request_time.as_millis() as u64);
        let f_request_time = request_time.clone();
        let r_request_time = request_time.clone();
        let config = Config {
            request_time: chain.config.request_time.as_millis() as u64,
            block_period: chain.config.block_period.as_secs(),
            chain_id: 0,
        };

        Core::create(move |ctx| {
            let core_pid = ctx.address().clone();
            let address = address.clone();
            let (f_core_pid, r_core_pid) = (core_pid.clone(), core_pid.clone());
            let b_core_pid = core_pid.clone();
            let mut backend = backend.clone();
            backend.set_core_pid(ctx.address());
            let core_backend: Box<Backend<ValidatorsType=ImplValidatorSet> + Send + Sync> = Box::new(backend.clone()) as Box<Backend<ValidatorsType=ImplValidatorSet> + Send + Sync>;

            Core {
                pid: ctx.address(),
                config: config,
                address: address,
                keypair: key_pair,
                state: State::AcceptRequest,
                validators: validators,

                current_state: current_state,
                round_change_set: round_change_set,
                wait_round_change: false,

                future_prepprepare_timer: Timer::create(move |_| {
                    Timer::new("future".to_owned(), f_request_time, f_core_pid, None)
                }),
                round_change_timer: Timer::create(move |_| {
                    Timer::new("round change".to_owned(), r_request_time, r_core_pid, None)
                }),

                consensus_timestamp: Duration::from_secs(0),

                backend: core_backend,

                backlog_store: BackLogActor::create(move |_| {
                    BackLogActor::new(b_core_pid)
                }),

                round_change_limiter: Instant::now(),
            }
        })
    }

    // p2p message
    fn handle_message(&mut self, payload: &[u8]) -> ConsensusResult {
        let mut msg: GossipMessage = GossipMessage::from_bytes(Cow::from(payload));
        let address = msg.address().map_err(|err| ConsensusError::Unknown(err))?;
        debug!("Message from {}", msg.trace());
        self.validators.get_by_address(address.clone()).ok_or(ConsensusError::UnauthorizedAddress)?;
        self.handle_check_message(&msg, &Validator::new(address))
    }

    fn handle_time_msg(&mut self) {
        if let Ok(last_proposal) = self.backend.last_proposal() {
            let last_block = last_proposal.block();
            if last_block.height() >= self.current_state.height() {
                trace!("round change timeout, catch up latest height, last_height: {}", last_block.height());
                return;
            }
            self.send_next_round_change();
        }
    }

    pub fn handle_check_message(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult {
        let result = match msg.code {
            MessageType::Preprepare => {
                <Core as HandlePreprepare>::handle(self, msg, src)
            }
            MessageType::Prepare => {
                <Core as HandlePrepare>::handle(self, msg, src)
            }
            MessageType::Commit => {
                <Core as HandleCommit>::handle(self, msg, src)
            }
            MessageType::RoundChange => {
                <Core as HandleRoundChange>::handle(self, msg, src)
            }
        };
        // TODO
        if let Err(ref err) = result {
            match err {
                ConsensusError::FutureMessage | ConsensusError::FutureRoundMessage => {
                    self.backlog_store.do_send(msg.clone());
                }
                _ => {}
            }
        }
        result
    }

    /// need to check：height，round，State
    /// if at waitting for change，should handle receive to fast consensus
    pub fn check_message(&self, code: MessageType, view: &View) -> Result<(), ConsensusError> {
        if view.height == 0 {
            return Err(ConsensusError::Unknown(
                "invalid view, height should be zero".to_string(),
            ));
        }

        if code == MessageType::RoundChange {
            // check view
            if view.height > self.current_state.height() {
                return Err(ConsensusError::FutureBlockMessage);
            } else if view.height < self.current_state.height() {
                return Err(ConsensusError::OldMessage);
            }
            return Ok(());
        }

        if view.height > self.current_state.height() {
            return Err(ConsensusError::FutureBlockMessage);
        }
        if view.height < self.current_state.height() {
            return Err(ConsensusError::OldMessage);
        }

        if self.state == State::AcceptRequest {
            if code > MessageType::Preprepare {
                return Err(ConsensusError::FutureMessage);
            }
            return Ok(());
        }

        Ok(())
    }

    // enter commit state
    pub fn commit(&mut self) {
        self.set_state(State::Committed);
        let mut committed_seals = Vec::with_capacity(self.current_state.commits.len());
        self.current_state.commits.values().iter().for_each(|v| {
            committed_seals.push(v.signature.as_ref().unwrap().clone());
        });
        let has_more_than_maj23 =
            self.validators.two_thirds_majority() + 1 <= committed_seals.len();
        assert!(has_more_than_maj23);
        // TODO commit
        let mut proposal = self.current_state.proposal().unwrap().clone();
        if let Err(err) = self.backend.commit(&mut proposal, committed_seals) {
            error!("Failed to commit block");
        }

        debug!(
            "commit proposal, hash:{}, height:{}",
            proposal.block().hash().short(),
            proposal.block().height()
        );
    }

    // TODO do more things
    pub fn finalize_message(&self, msg: &mut GossipMessage) -> Result<(), String> {
        msg.address = self.address.clone();
        msg.set_sign(&self.keypair.secret());
        Ok(())
    }

    pub fn broadcast(&mut self, msg: &GossipMessage) {
        let mut copy_msg = msg.clone();
        self.finalize_message(&mut copy_msg).unwrap();
        if let Err(err) = self.backend.gossip(&self.validators, copy_msg) {
            error!("Failed to gossip message, err: {:?}", err);
        }
    }

    // 启动新的轮次，触发的条件
    // 1：新高度初始化
    pub(crate) fn start_new_zero_round(&mut self) {
        trace!("before start zero round");
        let last_proposal = self.backend.last_proposal().unwrap();
        let last_height = last_proposal.block().height();
        // TODO 增加判断，last_proposal == blockend.proposal_hash
        let new_view: View = View::new(last_height + 1, 0);
        // TODO 从backend 获取 backend.validator_set
        self.validators = self.backend.validators(last_height + 1).clone();
        self.round_change_set = RoundChangeSet::new(self.validators.clone(), None);
        assert_ne!(
            self.validators.size(),
            0,
            "validators'size should be more than zero"
        );

        // New snapshot for new round
        self.update_round_state(new_view, self.validators.clone(), false);
        // calc new proposer
        self.validators
            .calc_proposer(&last_proposal.block().hash(), last_height, new_view.round);

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
    // 锁定+2/3的 round change 票
    pub(crate) fn start_new_round(&mut self, round: Round, _pre_change_prove: &[u8]) {
        trace!("before start new round");
        assert_ne!(
            round, 0,
            "zero round only call by self.start_new_zero_round"
        );
        let expect = round > self.current_state.round();
        assert!(
            expect,
            "new round should not be smaller than or equal current round"
        );
        let last_proposal = self.backend.last_proposal().unwrap();
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


        assert_ne!(
            self.validators.size(),
            0,
            "validators'size should be more than zero"
        );

        // TODO may try to chgeck
        // start new round timer
        //        let round = self.round_change_set
        //        .max_round(self.validators.two_thirds_majority() + 1).unwrap();
        trace!("ready to update round, because round change");
        let new_view = View::new(self.current_state.height(), round);

        // TODO 继承上一次的Round change prove
        // round change
        // TODO prove tree
        self.round_change_set = RoundChangeSet::new(self.validators.clone(), None);

        // New snapshot for new round
        self.update_round_state(new_view, self.validators.clone(), true);
        // calc new proposer
        self.validators
            .calc_proposer(&last_proposal.block().hash(), last_height, new_view.round);

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
                let proposal = self.current_state.pending_request.as_ref().unwrap();
                self.send_preprepare(&CSRequest::new(proposal.proposal.clone()));
            }
        }

        // reset new round change timer
        self.new_round_change_timer();
        info!(
            "after start new round, new round: {}",
            self.current_state.round()
        );
    }

    // 处理新的round
    // 等待+2/3
    pub(crate) fn catchup_round(&mut self, round: Round) {
        trace!(
            "catchup new round, current round:{}, new round: {}",
            self.current_state.round(),
            round
        );
        // set curret state into "wait for round change"
        self.wait_round_change = true;
        // start new round timer
        self.new_round_change_timer();
    }

    // TODO 修复不同节点锁定的提案不一致时，需要采用某种手段去修复
    // 如以锁定的周期最新为基点
    pub(crate) fn update_round_state(
        &mut self,
        view: View,
        vals: ImplValidatorSet,
        round_change: bool,
    ) {
        debug!("update round state");
        // 来自于轮次的改变
        if round_change {
            // 已经锁定在某一个高度，则应该继承其锁，且下一轮次继续以锁定的提案进行`共识`
            if self.current_state.is_locked() {
                self.current_state = RoundState::new_round_state(
                    view,
                    vals,
                    self.current_state.get_lock_hash(),
                    self.current_state.preprepare.clone(),
                    self.current_state.pending_request.take(),
                );
            } else {
                // 未锁定到某个提案
                self.current_state = RoundState::new_round_state(
                    view,
                    vals,
                    None,
                    None,
                    self.current_state.pending_request.take(),
                );
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

    pub fn is_proposer(&self) -> bool {
        self.validators.is_proposer(self.backend.address())
    }

    pub fn current_view(&self) -> View {
        View::new(self.current_state.height(), self.current_state.round())
    }

    pub fn mut_current_state(&mut self) -> &mut RoundState {
        &mut self.current_state
    }

    pub fn val_set(&self) -> &ImplValidatorSet {
        &self.validators
    }

    pub fn stop_future_preprepare_timer(&mut self) {
        // stop old timer
        self.future_prepprepare_timer.try_send(Op::Stop);
    }

    pub fn stop_round_change_timer(&mut self) {
        self.round_change_timer.try_send(Op::Stop);
        info!("stop round change timer");
    }

    pub fn stop_timer(&mut self) {
        self.stop_future_preprepare_timer();
        self.stop_round_change_timer();
    }

    pub fn new_round_change_timer(&mut self) {
        trace!("start new round timer");
        // stop old timer
        self.round_change_timer.try_send(Op::Stop);
        // start new timer
        let pid = self.pid.clone();
        self.round_change_timer = Timer::create(move |_| {
            Timer::new(
                "round change".to_string(),
                Duration::from_millis(3 * 1000),
                pid,
                None,
            )
        })
    }

    pub fn new_round_future_preprepare_timer(&mut self, duraton: Duration, msg: GossipMessage) {
        trace!("stop future preprepare timer");
        self.stop_future_preprepare_timer();
        let pid = self.pid.clone();
        self.future_prepprepare_timer =
            Timer::create(move |_| Timer::new("future preprepare".to_string(), duraton, pid, Some(msg)));
    }
}

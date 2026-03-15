use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::ethkey::Address;
use cryptocurrency_kit::ethkey::KeyPair;
use libp2p::PeerId;
use tokio::sync::mpsc;

use crossbeam::channel::Receiver as CrossbeamReceiver;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use std::sync::Arc;

use super::{
    preprepare::HandlePreprepare,
    prepare::HandlePrepare,
    commit::HandleCommit,
    round_change::HandleRoundChange,
    round_change_set::RoundChangeSet,
    round_state::RoundState,
    runner::{CoreHandle, CoreMessage},
};
use crate::{
    core::chain::Chain,
    consensus::validator::fn_selector,
    consensus::backend::{Backend, ImplBackend},
    consensus::config::Config,
    consensus::error::{ConsensusError, ConsensusResult},
    consensus::events::{OpCMD, MessageEvent, NewHeaderEvent, FinalCommittedEvent, BackLogEvent, TimerEvent},
    consensus::types::{Proposal, Request as CSRequest, Round, View},
    consensus::validator::{ImplValidatorSet, ValidatorSet},
    p2p::protocol::{RawMessage, P2PMsgCode},
    protocol::{GossipMessage, MessageType, State},
    types::Validator,
    types::block::Blocks,
    types::Height,
    subscriber::events::ChainEvent,
};

pub fn handle_msg_middle(core_handle: CoreHandle, chain: Arc<Chain>) -> impl Fn(PeerId, RawMessage) -> Result<(), String> + Clone {
    move |peer_id: PeerId, msg: RawMessage| {
        let header = msg.header();
        let payload = msg.payload().to_vec();
        match header.code {
            P2PMsgCode::Consensus => {
                core_handle.send_message(payload.clone());
                // Note: FutureBlockMessage retry is handled inside Core; message is processed async
            }
            P2PMsgCode::Block => {
                let blocks: Blocks = Blocks::from_bytes(Cow::from(&payload));
                debug!("Receive a batch block from network, size:{:?}", blocks.0.len());
                // TODO FIXME
                blocks.0.iter().for_each(|block| {
                    chain.insert_block(block);
                });
            }
            P2PMsgCode::Sync => {
                let height = Height::from_bytes(Cow::from(&payload));
                debug!("Receive a new sync event from network, height: {:?}", height);

                let last_height = chain.get_last_height();
                let mut total = 0;
                let mut batch = 0;
                let mut blocks = Blocks(vec![]);
                for height in height..last_height + 1  {
                    if let Some(block) = chain.get_block_by_height(height) {
                        blocks.0.push(block);
                    }
                    if batch > 20 {
                        chain.post_event(ChainEvent::PostBlock(Some(peer_id), blocks.clone()));
                        batch = 0;
                        // FIXME
                        blocks.0.clear();
                    }
                    if total > 100 {
                        break;
                    }
                    batch += 1;
                    total += 1;
                }
                if !blocks.0.is_empty() {
                    chain.post_event(ChainEvent::PostBlock(Some(peer_id), blocks));
                }
            }
            _ => unimplemented!()
        }

        Ok(())
    }
}


/// Core consensus - run loop only (actix Actor removed)
pub struct Core;

// --- CoreState: tokio-based Core without actix ---

/// Core state for tokio run loop - same fields as Core but with tokio timer/backlog
pub struct CoreState {
    pub config: Config,
    address: Address,
    pub keypair: KeyPair,
    pub state: State,
    validators: ImplValidatorSet,
    pub current_state: RoundState,
    pub round_change_set: RoundChangeSet<ImplValidatorSet>,
    pub wait_round_change: bool,
    pub consensus_timestamp: Duration,
    backlog_store: HashMap<Address, Vec<GossipMessage>>,
    pub backend: Box<dyn Backend<ValidatorsType = ImplValidatorSet>>,
    pub round_change_limiter: Instant,
    chain: Arc<Chain>,

    core_handle: CoreHandle,
    round_change_timer_handle: Option<tokio::task::JoinHandle<()>>,
    future_preprepare_timer_handle: Option<tokio::task::JoinHandle<()>>,
}

impl CoreState {
    fn add_to_backlog(&mut self, msg: GossipMessage) {
        self.backlog_store
            .entry(msg.address)
            .or_default()
            .push(msg);
    }

    fn stop_timer(&mut self) {
        if let Some(h) = self.round_change_timer_handle.take() {
            h.abort();
        }
        if let Some(h) = self.future_preprepare_timer_handle.take() {
            h.abort();
        }
    }

    pub(crate) fn new_round_change_timer(&mut self) {
        if let Some(h) = self.round_change_timer_handle.take() {
            h.abort();
        }
        let handle = self.core_handle.clone();
        let timeout_ms = self.config.request_time;
        self.round_change_timer_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
            handle.send_timer();
        }));
    }

    pub(crate) fn new_round_future_preprepare_timer(&mut self, duration: Duration, msg: GossipMessage) {
        if let Some(h) = self.future_preprepare_timer_handle.take() {
            h.abort();
        }
        let handle = self.core_handle.clone();
        self.future_preprepare_timer_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            handle.send_backlog(msg);
        }));
    }

    #[allow(dead_code)]
    fn stop_future_preprepare_timer(&mut self) {
        if let Some(h) = self.future_preprepare_timer_handle.take() {
            h.abort();
        }
    }

    pub(crate) fn address(&self) -> Address {
        self.address
    }

    pub(crate) fn val_set(&self) -> &ImplValidatorSet {
        &self.validators
    }

    pub(crate) fn current_view(&self) -> View {
        View::new(self.current_state.height(), self.current_state.round())
    }

    pub(crate) fn set_state(&mut self, new_state: State) {
        trace!("state change, from {:?} to {:?}", self.state, new_state);
        self.state = new_state;
    }

    #[allow(dead_code)]
    fn mut_current_state(&mut self) -> &mut RoundState {
        &mut self.current_state
    }

    pub(crate) fn is_proposer(&self) -> bool {
        self.validators.is_proposer(self.backend.address())
    }

    fn handle_message(&mut self, payload: &[u8]) -> ConsensusResult {
        if self.val_set().size() == 0 {
            return Ok(());
        }
        let mut msg: GossipMessage = GossipMessage::from_bytes(Cow::from(payload));
        let address = msg.address().map_err(ConsensusError::Unknown)?;
        debug!("Message from {}", msg.trace());
        self.validators
            .get_by_address(address)
            .ok_or(ConsensusError::UnauthorizedAddress)?;
        self.handle_check_message(&msg, &Validator::new(address))
    }

    fn handle_check_message(&mut self, msg: &GossipMessage, src: &Validator) -> ConsensusResult {
        let result = match msg.code {
            MessageType::Preprepare => <CoreState as HandlePreprepare>::handle(self, msg, src),
            MessageType::Prepare => <CoreState as HandlePrepare>::handle(self, msg, src),
            MessageType::Commit => <CoreState as HandleCommit>::handle(self, msg, src),
            MessageType::RoundChange => <CoreState as HandleRoundChange>::handle(self, msg, src),
        };
        if let Err(ref err) = result {
            match err {
                ConsensusError::FutureMessage | ConsensusError::FutureRoundMessage => {
                    self.add_to_backlog(msg.clone());
                }
                _ => {}
            }
        }
        result
    }

    pub(crate) fn check_message(&self, code: MessageType, view: &View) -> Result<(), ConsensusError> {
        if view.height == 0 {
            return Err(ConsensusError::Unknown(
                "invalid view, height should be zero".to_string(),
            ));
        }
        if code == MessageType::RoundChange {
            if view.height > self.current_state.height() {
                return Err(ConsensusError::FutureBlockMessage(view.height));
            } else if view.height < self.current_state.height() {
                return Err(ConsensusError::OldMessage);
            }
            return Ok(());
        }
        if view.height > self.current_state.height() {
            return Err(ConsensusError::FutureBlockMessage(view.height));
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

    fn finalize_message(&self, msg: &mut GossipMessage) -> Result<(), String> {
        msg.address = self.address;
        msg.set_sign(self.keypair.secret());
        Ok(())
    }

    pub(crate) fn broadcast(&mut self, msg: &GossipMessage) {
        let mut copy_msg = msg.clone();
        self.finalize_message(&mut copy_msg).unwrap();
        if let Err(err) = self.backend.gossip(&self.validators, copy_msg) {
            error!("Failed to gossip message, err: {:?}", err);
        }
    }

    fn update_round_state(
        &mut self,
        view: View,
        vals: ImplValidatorSet,
        round_change: bool,
    ) {
        if round_change {
            if self.current_state.is_locked() {
                self.current_state = RoundState::new_round_state(
                    view,
                    vals,
                    self.current_state.get_lock_hash(),
                    self.current_state.preprepare.clone(),
                    self.current_state.pending_request.take(),
                );
            } else {
                self.current_state = RoundState::new_round_state(
                    view,
                    vals,
                    None,
                    None,
                    self.current_state.pending_request.take(),
                );
            }
        } else {
            self.current_state = RoundState::new_round_state(view, vals, None, None, None);
        }
    }

    fn start_new_zero_round(&mut self) {
        trace!("before start zero round");
        let last_proposal = self.backend.last_proposal().unwrap();
        let last_height = last_proposal.block().height();
        let new_view = View::new(last_height + 1, 0);
        self.validators = self.backend.validators(last_height + 1).clone();
        self.round_change_set = RoundChangeSet::new(self.validators.clone(), None);
        assert_ne!(self.validators.size(), 0, "validators'size should be more than zero");

        self.update_round_state(new_view, self.validators.clone(), false);
        self.validators
            .calc_proposer(&last_proposal.block().hash(), last_height, new_view.round);

        self.wait_round_change = false;
        self.set_state(State::AcceptRequest);
        self.new_round_change_timer();
        debug!("after start zero round");
    }

    pub(crate) fn start_new_round(&mut self, round: Round, _pre_change_prove: &[u8]) {
        trace!("before start new round");
        assert_ne!(round, 0, "zero round only call by self.start_new_zero_round");
        assert!(
            round > self.current_state.round(),
            "new round should not be smaller than or equal current round"
        );
        let last_proposal = self.backend.last_proposal().unwrap();
        let last_height = last_proposal.block().height();
        if last_height > self.current_state.height() {
            trace!("catchup latest proposal, it should be not happen");
            return;
        }
        assert_ne!(self.validators.size(), 0, "validators'size should be more than zero");

        let new_view = View::new(self.current_state.height(), round);
        self.round_change_set = RoundChangeSet::new(self.validators.clone(), None);

        self.update_round_state(new_view, self.validators.clone(), true);
        self.validators
            .calc_proposer(&last_proposal.block().hash(), last_height, new_view.round);

        self.wait_round_change = false;
        self.set_state(State::AcceptRequest);

        if self.validators.is_proposer(self.address) {
            if self.current_state.is_locked() {
                let r = CSRequest::new(self.current_state.proposal().unwrap().clone());
                self.send_preprepare(&r);
            } else if let Some(ref proposal) = self.current_state.pending_request {
                self.send_preprepare(&CSRequest::new(proposal.proposal.clone()));
            }
        }

        self.new_round_change_timer();
        debug!("after start new round, new round: {}", self.current_state.round());
    }

    pub(crate) fn catchup_round(&mut self, round: Round) {
        trace!(
            "catchup new round, current round:{}, new round: {}",
            self.current_state.round(),
            round
        );
        self.wait_round_change = true;
        self.new_round_change_timer();
    }

    pub(crate) fn commit(&mut self) {
        self.set_state(State::Committed);
        let mut committed_seals = Vec::with_capacity(self.current_state.commits.len());
        self.current_state.commits.values().iter().for_each(|v| {
            committed_seals.push(v.signature.as_ref().unwrap().clone());
        });
        let has_more_than_maj23 =
            self.validators.two_thirds_majority() < committed_seals.len();
        assert!(has_more_than_maj23);
        let mut proposal = self.current_state.proposal().unwrap().clone();
        if let Err(_err) = self.backend.commit(&mut proposal, committed_seals) {
            error!("Failed to commit block");
        }
        debug!(
            "commit proposal, hash:{}, height:{}",
            proposal.block().hash().short(),
            proposal.block().height()
        );
    }

    fn handle_new_header(&mut self, msg: NewHeaderEvent) -> ConsensusResult {
        debug!("Receive a new header event");
        let proposal = msg.proposal.clone();
        self.start_new_zero_round();
        self.check_request_message(&crate::consensus::types::Request::new(proposal.clone()))?;
        assert_eq!(self.state, State::AcceptRequest);
        self.accept(&crate::consensus::types::Request::new(proposal.clone()));
        self.send_preprepare(&crate::consensus::types::Request::new(proposal));
        Ok(())
    }

    fn check_request_message(&self, request: &crate::consensus::types::Request<Proposal>) -> ConsensusResult {
        if self.current_state.height() == 0 {
            return Err(ConsensusError::WaitNewRound);
        }
        if self.current_state.height() > request.proposal.block().height() {
            return Err(ConsensusError::OldMessage);
        }
        if self.current_state.height() < request.proposal.block().height() {
            return Err(ConsensusError::FutureMessage);
        }
        Ok(())
    }

    fn accept(&mut self, request: &crate::consensus::types::Request<Proposal>) {
        self.current_state.pending_request = Some(crate::consensus::types::Request {
            proposal: request.proposal.clone(),
        });
    }

    fn handle_final_committed(&mut self, _msg: FinalCommittedEvent) {
        self.stop_timer();
        self.wait_round_change = false;
    }

    fn handle_message_event(&mut self, msg: MessageEvent) -> ConsensusResult {
        let result = self.handle_message(&msg.payload);
        if let Err(ref err) = result {
            match err {
                e @ ConsensusError::FutureBlockMessage(_) => debug!("Failed to handle message, err: {:?}", e),
                e @ ConsensusError::OldMessage
                | e @ ConsensusError::FutureRoundMessage
                | e @ ConsensusError::FutureMessage
                | e @ ConsensusError::NotFromProposer => debug!("Failed to handle message, err: {:?}", e),
                other => error!("Failed to handle message, err: {:?}", other),
            }
        }
        result
    }

    fn handle_backlog_event(&mut self, msg: BackLogEvent) -> ConsensusResult {
        let src = Validator::new(msg.msg.address);
        self.handle_check_message(&msg.msg, &src)
    }

    fn handle_timer_event(&mut self, _msg: TimerEvent) {
        debug!("Receive timer event");
        let last_proposal = self.backend.last_proposal().unwrap();
        let last_block = last_proposal.block();
        let cur_view = self.current_view();
        if last_block.height() >= cur_view.height {
            debug!("Round change timeout, catch up latest height");
            self.stop_timer();
            self.wait_round_change = false;
        } else {
            self.send_next_round_change();
        }
    }

    fn handle_op_cmd(&mut self, msg: OpCMD) -> bool {
        match msg {
            OpCMD::Stop => {
                self.stop_timer();
                true
            }
            OpCMD::Ping => {
                debug!("Recive a test message");
                false
            }
        }
    }
}

impl Core {
    /// Async run loop - replaces actix Actor. Creates CoreState and processes CoreMessage.
    /// Uses a bridge thread to convert crossbeam Receiver to tokio mpsc for async recv.
    pub async fn run(
        chain: Arc<Chain>,
        backend: ImplBackend,
        key_pair: KeyPair,
        core_rx: CrossbeamReceiver<CoreMessage>,
        core_handle: CoreHandle,
    ) {
        let (bridge_tx, mut rx) = mpsc::unbounded_channel();
        tokio::task::spawn_blocking(move || {
            while let Ok(msg) = core_rx.recv() {
                if bridge_tx.send(msg).is_err() {
                    break;
                }
            }
        });
        let mut backend = backend;
        backend.set_core_handle(core_handle.clone());
        let core_backend: Box<dyn Backend<ValidatorsType = ImplValidatorSet> + Send + Sync> =
            Box::new(backend) as Box<dyn Backend<ValidatorsType = ImplValidatorSet> + Send + Sync>;

        let address = key_pair.address();
        let last_block = chain.get_last_block();
        let validators = chain.get_validators(last_block.height());
        let addresses: Vec<Address> = validators.iter().map(|v| *v.address()).collect();
        let validators = ImplValidatorSet::new(&addresses, Box::new(fn_selector));

        let last_view = View::new(last_block.height(), 0);
        let lock_hash = last_block.hash();
        let current_state = RoundState::new_round_state(
            last_view,
            validators.clone(),
            Some(lock_hash),
            None,
            None,
        );
        let round_change_set = RoundChangeSet::new(validators.clone(), None);

        let config = Config {
            request_time: chain.config.request_time.as_millis() as u64,
            block_period: chain.config.block_period.as_secs(),
            chain_id: 0,
        };

        let mut state = CoreState {
            config,
            address,
            keypair: key_pair,
            state: State::AcceptRequest,
            validators,
            current_state,
            round_change_set,
            wait_round_change: false,
            consensus_timestamp: Duration::from_secs(0),
            backlog_store: HashMap::new(),
            backend: core_backend,
            round_change_limiter: Instant::now(),
            chain: chain.clone(),
            core_handle,
            round_change_timer_handle: None,
            future_preprepare_timer_handle: None,
        };

        state.start_new_zero_round();

        info!("core run loop started");

        let mut msg_count: u64 = 0;
        while let Some(msg) = rx.recv().await {
            msg_count += 1;
            match msg {
                CoreMessage::Message(m) => {
                    trace!("core msg #{} Message", msg_count);
                    if let Err(ref e) = state.handle_message_event(m) {
                        if let ConsensusError::FutureBlockMessage(height) = e {
                            let height = *height;
                            let chain = state.chain.clone();
                            tokio::spawn(async move {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                let last_height = chain.get_last_height();
                                if last_height < height {
                                    chain.post_event(ChainEvent::SyncBlock(last_height + 1));
                                }
                            });
                        }
                        debug!("handle_message_event err: {:?}", e);
                    }
                }
                CoreMessage::NewHeader(m) => {
                    trace!("core msg #{} NewHeader height={}", msg_count, m.proposal.block().height());
                    if let Err(ref e) = state.handle_new_header(m) {
                        debug!("handle_new_header err: {:?}", e);
                    }
                }
                CoreMessage::FinalCommitted(m) => {
                    trace!("core msg #{} FinalCommitted", msg_count);
                    state.handle_final_committed(m)
                }
                CoreMessage::BackLog(m) => {
                    trace!("core msg #{} BackLog", msg_count);
                    if let Err(ref e) = state.handle_backlog_event(m) {
                        debug!("handle_backlog_event err: {:?}", e);
                    }
                }
                CoreMessage::Timer(m) => {
                    trace!("core msg #{} Timer", msg_count);
                    state.handle_timer_event(m)
                }
                CoreMessage::Op(op) => {
                    if state.handle_op_cmd(op) {
                        break;
                    }
                }
            }
        }

        info!("core run loop stopped");
    }
}

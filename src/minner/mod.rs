use std::sync::Arc;

use crossbeam::channel;
use parking_lot::RwLock;
use rand::random;
use cryptocurrency_kit::ethkey::{Address, KeyPair};
use cryptocurrency_kit::crypto::Hash;

use crate::{
    subscriber::events::ChainEvent,
    core::chain::Chain,
    core::tx_pool::SafeTxPool,
    consensus::consensus::SafeEngine,
    types::block::{Block, Header},
    types::transaction::{Transaction, merkle_root_transactions},
};

/// Start the minner in a dedicated thread - subscribes to ChainEventBus and mines blocks
pub fn start_minner(
    _config: &crate::config::Config,
    key_pair: KeyPair,
    chain: Arc<Chain>,
    _txpool: Arc<RwLock<SafeTxPool>>,
    mut engine: SafeEngine,
) {
    let minter = key_pair.address();
    let chain_bus = chain.chain_event_bus();

    std::thread::spawn(move || {
        info!("Start minner");
        chain.post_event(ChainEvent::SyncBlock(chain.get_last_height() + 1));

        loop {
            let mut block = packet_next_block(minter, &key_pair, &chain);
            let mint_height = block.height();

            let (abort_tx, abort_rx) = channel::bounded(1);
            let mut chain_rx = chain_bus.subscribe();

            let abort_tx_clone = abort_tx.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("runtime");
                rt.block_on(async {
                    while let Ok(event) = chain_rx.recv().await {
                        if let ChainEvent::NewHeader(last_header) = event {
                            if last_header.height >= mint_height {
                                let _ = abort_tx_clone.send(());
                                break;
                            }
                        }
                    }
                });
            });

            if let Err(err) = engine.seal(&mut block, abort_rx) {
                info!("Failed to seal consensus, err: {:?}", err);
            }
        }
    });
}

fn packet_next_block(minter: Address, key_pair: &KeyPair, chain: &Chain) -> Block {
    let (next_time, pre_header) = next_block(chain);
    let coinbase = coinbase_transaction(minter, key_pair, chain);

    let pre_hash: Hash = pre_header.block_hash();
    let tx_hash = merkle_root_transactions(vec![coinbase.clone()]);
    let extra = Vec::from("Coinse base");

    let mut header = Header::new_mock(
        pre_hash,
        minter,
        tx_hash,
        pre_header.height + 1,
        next_time,
        Some(extra),
    );
    header.cache_hash(None);
    Block::new(header, vec![coinbase])
}

fn coinbase_transaction(minter: Address, key_pair: &KeyPair, chain: &Chain) -> Transaction {
    let nonce: u64 = random();
    let to = minter;
    let amount = random::<u64>();
    let gas_limit = random::<u64>();
    let gas_price = 1_u64;
    let payload = Vec::from(chrono::Local::now().to_string());

    let mut transaction = Transaction::new(nonce, to, amount, gas_limit, gas_price, payload);
    transaction.sign(chain.config.chain_id, key_pair.secret());
    transaction
}

fn next_block(chain: &Chain) -> (u64, Header) {
    let pre_block = chain.get_last_block();
    let pre_header = pre_block.header();
    let pre_timestamp = pre_header.time;
    let next_timestamp = pre_timestamp + chain.config.block_period.as_secs();
    let now_timestamp = chrono::Local::now().timestamp() as u64;
    trace!(
        "now timestamp: {}, pre_timestamp: {}, next_timestamp: {}",
        now_timestamp,
        pre_timestamp,
        next_timestamp
    );
    if now_timestamp > next_timestamp {
        return (now_timestamp, pre_header.clone());
    }
    (next_timestamp, pre_header.clone())
}

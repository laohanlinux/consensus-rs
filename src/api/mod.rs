use std::sync::Arc;

use crate::core::chain::Chain;
use crate::types::block::Blocks;

use http::StatusCode;
use tide::{body, head, configuration::{Configuration, Environment}, App, AppData};

async fn blocks(mut chain: AppData<Arc<Chain>>) -> String {
    let state: &Arc<Chain> = &chain.0;
    let last_height = state.get_last_height();
    let mut blocks: Blocks = Blocks(vec![]);
    (0..last_height + 1).for_each(|height| {
        let block_hash = state.get_block_hash_by_height(height).unwrap();
        let block = state.get_block_by_hash(&block_hash).unwrap();
        blocks.0.push(block);
    });
    serde_json::to_string(&blocks).unwrap()
}

async fn transactions(mut chain: AppData<Arc<Chain>>) -> String {
    let state: &Arc<Chain> = &chain.0;
    let mut transactions = state.get_transactions();
    serde_json::to_string(&transactions).unwrap()
}

pub fn start_api(chain: Arc<Chain>, ip: String, port: u16) {
    let mut app = App::new(chain);
    app.at("/blocks").get(blocks);
    app.at("/transactions").get(transactions);
    app.config(Configuration {
        env: Environment::Production,
        address: ip,
        port: port,
    });
    app.serve();
}
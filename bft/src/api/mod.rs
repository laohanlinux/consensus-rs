use actix_web::{http, server, App, State, Responder, HttpRequest, HttpResponse};
use futures::future;
use futures::Future;

use std::sync::Arc;
use crate::core::chain::Chain;
use crate::types::block::Blocks;

struct Context {
    chain: Arc<Chain>,
}

fn blocks(req: &HttpRequest<Context>) -> impl Responder {
    let state: &Arc<Chain> = &req.state().chain;
    let last_height = state.get_last_height();
    info!("last height {}", last_height);
    let mut blocks: Blocks = Vec::new();
    (0..last_height + 1).for_each(|height| {
        let block_hash = state.get_block_hash_by_height(height).unwrap();
        let block = state.get_block_by_hash(&block_hash).unwrap();
        blocks.push(block);
    });
    serde_json::to_string(&blocks).unwrap()
}

fn transactions(req: &HttpRequest<Context>) -> impl Responder {
    let state: &Arc<Chain> = &req.state().chain;
    let mut transactions = state.get_transactions();
    serde_json::to_string(&transactions).unwrap()
}

pub fn start_api(chain: Arc<Chain>, http_addr: String) {
    server::new(move || {
        let chain = chain.clone();
        App::with_state(Context { chain: chain })
            .resource("/blocks", |r| r.method(http::Method::GET).f(blocks))
            .resource("/transactions", |r| r.method(http::Method::GET).f(transactions))
    }).bind(&http_addr).expect(&format!("Can not bind to {}", &http_addr))
        .shutdown_timeout(20)
        .workers(2)
        .run()
}
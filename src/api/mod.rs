use std::net::SocketAddr;
use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};

use crate::core::chain::Chain;
use crate::types::block::Blocks;

async fn blocks(State(chain): State<Arc<Chain>>) -> Json<Blocks> {
    let last_height = chain.get_last_height();
    let mut blocks: Blocks = Blocks(vec![]);
    for height in 0..=last_height {
        if let Some(block_hash) = chain.get_block_hash_by_height(height) {
            if let Some(block) = chain.get_block_by_hash(&block_hash) {
                blocks.0.push(block);
            }
        }
    }
    Json(blocks)
}

async fn transactions(State(chain): State<Arc<Chain>>) -> Json<Vec<crate::types::transaction::Transaction>> {
    Json(chain.get_transactions())
}

pub fn start_api(chain: Arc<Chain>, ip: String, port: u16) {
    let addr: SocketAddr = format!("{}:{}", ip, port).parse().expect("invalid api address");

    let app = Router::new()
        .route("/blocks", get(blocks))
        .route("/transactions", get(transactions))
        .with_state(chain);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind API");
            axum::serve(listener, app).await.expect("API server error");
        });
    });
}

//! Minimal P2P mDNS discovery example using libp2p 0.53

use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, SwarmBuilder};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mdns_config = libp2p::mdns::Config {
        ttl: Duration::from_secs(60),
        query_interval: Duration::from_secs(60),
        enable_ipv6: false,
    };

    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(move |keypair| {
            libp2p::mdns::tokio::Behaviour::new(
                mdns_config.clone(),
                keypair.public().to_peer_id(),
            )
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })?
        .build();

    let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
    swarm.listen_on(listen_addr)?;

    println!("Local peer id: {}", swarm.local_peer_id());

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(libp2p::mdns::Event::Discovered(peers)) => {
                for (peer_id, addr) in peers {
                    println!("Discovered peer {} at {}", peer_id, addr);
                }
            }
            SwarmEvent::Behaviour(libp2p::mdns::Event::Expired(peers)) => {
                for (peer_id, addr) in peers {
                    println!("Peer expired {} at {}", peer_id, addr);
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {}", address);
            }
            _ => {}
        }
    }
}

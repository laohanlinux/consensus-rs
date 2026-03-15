use std::time::Duration;

use futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{Multiaddr, PeerId, SwarmBuilder};

use crate::subscriber::{P2PEvent, P2PEventBus};

/// Handle for the discover service - can be used to stop it
#[derive(Clone)]
pub struct DiscoverServiceHandle {
    _tx: tokio::sync::mpsc::Sender<()>,
}

pub struct DiscoverService;

impl DiscoverService {
    /// Run mDNS discovery in a background thread, sending P2PEvent::AddPeer to the bus
    pub fn run_discover_service(
        p2p_bus: P2PEventBus,
        local_peer_id: PeerId,
        local_address: Multiaddr,
        ttl: Duration,
    ) {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            rt.block_on(async move {
                let mdns_config = libp2p::mdns::Config {
                    ttl,
                    query_interval: Duration::from_secs(60),
                    enable_ipv6: false,
                };

                let mut swarm = match SwarmBuilder::with_new_identity()
                    .with_tokio()
                    .with_tcp(
                        libp2p::tcp::Config::default(),
                        libp2p::noise::Config::new,
                        libp2p::yamux::Config::default,
                    )
                {
                    Ok(builder) => match builder.with_behaviour(move |keypair| {
                        libp2p::mdns::tokio::Behaviour::new(
                            mdns_config.clone(),
                            keypair.public().to_peer_id(),
                        )
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                    }) {
                        Ok(b) => b.build(),
                        Err(e) => {
                            error!("Failed to create mdns behaviour: {:?}", e);
                            return;
                        }
                    },
                    Err(e) => {
                        error!("Failed to create TCP transport: {:?}", e);
                        return;
                    }
                };

                info!("Discover service starting, will listen on {:?}", local_address);
                if let Err(e) = swarm.listen_on(local_address) {
                    error!("Failed to listen on address: {:?}", e);
                    return;
                }

                loop {
                    match swarm.select_next_some().await {
                        SwarmEvent::Behaviour(libp2p::mdns::Event::Discovered(peers)) => {
                            for (peer_id, addr) in peers {
                                if peer_id != local_peer_id {
                                    info!("Discovered peer {} at {}", peer_id, addr);
                                    p2p_bus.send(P2PEvent::AddPeer(peer_id, vec![addr]));
                                }
                            }
                        }
                        SwarmEvent::Behaviour(libp2p::mdns::Event::Expired(peers)) => {
                            for (peer_id, addr) in peers {
                                debug!("Peer expired {} at {}", peer_id, addr);
                                p2p_bus.send(P2PEvent::DropPeer(peer_id, vec![addr]));
                            }
                        }
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Discover service listening on {}", address);
                        }
                        SwarmEvent::ConnectionClosed { .. }
                        | SwarmEvent::ConnectionEstablished { .. }
                        | SwarmEvent::OutgoingConnectionError { .. }
                        | SwarmEvent::IncomingConnection { .. }
                        | SwarmEvent::IncomingConnectionError { .. }
                        | SwarmEvent::ListenerClosed { .. }
                        | SwarmEvent::ListenerError { .. }
                        | SwarmEvent::Dialing { .. } => {}
                        other => {
                            trace!("Swarm event: {:?}", other);
                        }
                    }
                }
            });
        });
    }
}

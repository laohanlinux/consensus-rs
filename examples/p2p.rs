extern crate bft;
extern crate futures;
extern crate libp2p;
extern crate rand;
extern crate tokio;

use futures::prelude::*;
use libp2p::mdns::{MdnsPacket, MdnsService};
use libp2p::PeerId;
use libp2p::core::PublicKey;
use libp2p::multiaddr::{Multiaddr, ToMultiaddr};
use std::io;
use rand::Rng;
use std::time::Duration;

fn main() {
    let mut service = MdnsService::new().expect("Error while creating mDNS service");
    let my_peer_id = PeerId::random();
    let mut my_listened_addrs = Vec::new();
    println!("my pid {:?}", my_peer_id);
    let port = rand::random::<u8>();
    let address: Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap();


    my_listened_addrs.push(address);
    let future = futures::future::poll_fn(move || -> Poll<(), io::Error> {
        loop {
            // Grab the next available packet from the service.
            let packet = match service.poll() {
                Async::Ready(packet) => packet,
                Async::NotReady => return Ok(Async::NotReady),
            };

            match packet {
                MdnsPacket::Query(query) => {
                    // We detected a libp2p mDNS query on the network. In a real application, you
                    // probably want to answer this query by doing `query.respond(...)`.
                    println!("Detected query from {:?}", query.remote_addr());
                    query.respond(my_peer_id.clone(), my_listened_addrs.clone(), Duration::from_secs(3)).unwrap();
                }
                MdnsPacket::Response(response) => {
                    // We detected a libp2p mDNS response on the network. Responses are for
                    // everyone and not just for the requester, which makes it possible to
                    // passively listen.
                    for peer in response.discovered_peers() {
                        println!("Discovered peer {:?}", peer.id());
                        // These are the self-reported addresses of the peer we just discovered.
                        for addr in peer.addresses() {
                            println!(" Address = {:?}", addr);
                        }
                    }
                }
                MdnsPacket::ServiceDiscovery(query) => {
                    // The last possibility is a service detection query from DNS-SD.
                    // Just like `Query`, in a real application you probably want to call
                    // `query.respond`.
                    println!("Detected service query from {:?}", query.remote_addr());
                    query.respond(std::time::Duration::from_secs(120));
                }
            }
        }
    });

    // Blocks the thread until the future runs to completion (which will never happen).
    tokio::run(future.map_err(|err| panic!("{:?}", err)));
}

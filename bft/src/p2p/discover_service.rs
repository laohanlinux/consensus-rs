use std::io::{self, Write};
use std::time::Duration;

#[macro_use]
use actix::prelude::*;
use futures::future;
use futures::prelude::*;
use libp2p::core::PublicKey;
use libp2p::mdns::{MdnsPacket, MdnsService};
use libp2p::multiaddr::{Multiaddr, ToMultiaddr};
use libp2p::PeerId;

#[macro_use]
use subscriber::*;

pub struct DiscoverService {
    p2p_pid: Addr<ProcessSignals>,
}

impl Actor for DiscoverService {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        use std::io::{self, Write};
        writeln!(io::stdout(), "discover service started");
    }
}

impl DiscoverService {
    pub fn spawn_discover_service(
        p2p_subscriber: Addr<ProcessSignals>,
        peer_id: PeerId,
        local_address: Multiaddr,
        ttl: Duration,
    ) -> Addr<DiscoverService> {
        let mut service = MdnsService::new().expect("Error while creating mDNS service");
        let p2p_subscriber_clone = p2p_subscriber.clone();
        let future = futures::future::poll_fn(move || -> Poll<(), io::Error> {
            loop {
                let packet = match service.poll() {
                    Async::Ready(packet) => packet,
                    Async::NotReady => return Ok(Async::NotReady),
                };
                match packet {
                    MdnsPacket::Query(query) => {
                        query
                            .respond(peer_id.clone(), vec![local_address.clone()], ttl)
                            .unwrap();
                    }
                    MdnsPacket::Response(response) => {
                        for peer in response.discovered_peers() {
                            writeln!(io::stdout(), "mdsc packet, local->{:?}, remote->{:?}", peer_id.clone().to_base58(), peer.id().to_base58());
                            let id = peer.id().clone();
                            if peer_id.clone() == id {
                                continue;
                            }
                            let mut addresses: Vec<Multiaddr> = Vec::new();
                            for address in peer.addresses() {
                                addresses.push(address.clone());
                            }
                            let request = p2p_subscriber_clone.send(P2PEvent::AddPeer(id, addresses));
                            Arbiter::spawn(request.then(|_|{future::result(Ok(()))}));
                        }
                    }
                    MdnsPacket::ServiceDiscovery(query) => {
                        query.respond(ttl);
                    }
                }
            }
        });

        Arbiter::spawn(future.then(|res| {
            writeln!(io::stdout(), "Got request");
            future::result(Ok(()))
        }));

        let p2p_subscriber = p2p_subscriber.clone();
        Actor::create(|_| DiscoverService {
            p2p_pid: p2p_subscriber,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::io::{self, Write};

    pub struct Ping {}

    impl Message for Ping {
        type Result = ();
    }

    type PongRecipient = Recipient<Ping>;

    type PongRecipients<T: Message> = Vec<Recipient<T>>;

    struct Worker {}

    impl Actor for Worker {
        type Context = Context<Self>;
    }

    impl Handler<P2PEvent> for Worker {
        type Result = ();
        fn handle(&mut self, msg: P2PEvent, _: &mut Self::Context) {
            match msg {
                P2PEvent::AddPeer(_, _) => {
                    writeln!(io::stdout(), "work receive a msg: {:?}", msg);
                }
                P2PEvent::DropPeer(_, _) => {
                    writeln!(io::stdout(), "work receive a msg: {:?}", msg);
                }
            }
        }
    }

    #[test]
    fn t_discover_service() {
        let system = System::new("test");
        (0..10).for_each(|_|{
            let peer_id = PeerId::random();
            let port = rand::random::<u8>();
            let address: Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap();
            let p2p_subscriber = spawn_sync_subscriber();
            // register
            let recipient = p2p_subscriber.clone().recipient();
            let message = SubscribeMessage::SubScribe(recipient);
            let request = p2p_subscriber.send(message);
            Arbiter::spawn(request.then(|_|{
                writeln!(io::stdout(), "Got request, Hi").unwrap();
                future::result(Ok(()))
            }));
            let pid = DiscoverService::spawn_discover_service(p2p_subscriber.clone(), peer_id, address, Duration::from_secs(3));
        });
        system.run();
    }
}

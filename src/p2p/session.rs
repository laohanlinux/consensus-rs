use std::time::Duration;

use cryptocurrency_kit::storage::values::StorageValue;
use cryptocurrency_kit::crypto::Hash;
use futures::{SinkExt, StreamExt};
use libp2p::PeerId;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::interval;
use tokio_util::codec::{FramedRead, FramedWrite};

use super::codec::MsgPacketCodec;
use super::protocol::{BoundType, RawMessage, Header, P2PMsgCode, Handshake};
use super::server::{ServerEvent, TcpServerHandle};

pub type SessionTx = tokio::sync::mpsc::UnboundedSender<RawMessage>;

pub struct Session {
    peer_id: PeerId,
    local_id: PeerId,
    server: TcpServerHandle,
    bound_type: BoundType,
    handshaked: bool,
    genesis: Hash,
    write_tx: SessionTx,
}

impl Session {
    pub fn new(
        peer_id: PeerId,
        local_id: PeerId,
        server: TcpServerHandle,
        bound_type: BoundType,
        genesis: Hash,
        write_tx: SessionTx,
    ) -> Self {
        Session {
            peer_id,
            local_id,
            server,
            bound_type,
            handshaked: false,
            genesis,
            write_tx,
        }
    }

    pub async fn run(
        mut self,
        read: impl AsyncRead + Unpin,
        write: impl AsyncWrite + Unpin,
        mut write_rx: tokio::sync::mpsc::UnboundedReceiver<RawMessage>,
    ) {
        let mut framed_read = FramedRead::new(read, MsgPacketCodec);
        let mut framed_write = FramedWrite::new(write, MsgPacketCodec);

        // Send handshake
        let handshake =
            Handshake::new("0.1.1".to_string(), self.local_id, self.genesis);
        let raw_message = RawMessage::new(
            Header::new(
                P2PMsgCode::Handshake,
                10,
                chrono::Local::now().timestamp_nanos() as u64,
                None,
            ),
            handshake.into_bytes(),
        );
        if framed_write.send(raw_message).await.is_err() {
            return;
        }

        let mut ping_interval = interval(Duration::from_secs(1));
        let mut handshake_timeout = tokio::time::interval(Duration::from_secs(1));
        handshake_timeout.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                msg = framed_read.next() => {
                    match msg {
                        Some(Ok(raw_msg)) => {
                            if self.handle_message(raw_msg, &mut framed_write).await.is_err() {
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            debug!("Session read error: {:?}", e);
                            break;
                        }
                        None => break,
                    }
                }
                msg = write_rx.recv() => {
                    if let Some(raw_msg) = msg {
                        if raw_msg.header().code != P2PMsgCode::Ping {
                            debug!("Write message: {:?}, local_id:{:?}, peer_id:{:?}", raw_msg.header(), self.local_id.to_base58(), self.peer_id.to_base58());
                        }
                        if framed_write.send(raw_msg).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                _ = ping_interval.tick() => {
                    if self.handshaked {
                        let ping_msg = RawMessage::new(
                            Header::new(P2PMsgCode::Ping, 3, chrono::Local::now().timestamp_millis() as u64, None),
                            vec![],
                        );
                        if framed_write.send(ping_msg).await.is_err() {
                            break;
                        }
                    }
                }
                _ = handshake_timeout.tick() => {
                    if !self.handshaked {
                        trace!("Handshake timeout, local_id: {}, peer: {}", self.local_id.to_base58(), self.peer_id.to_base58());
                        break;
                    }
                }
            }
        }

        if self.handshaked {
            self.server.try_send(ServerEvent::Disconnected(self.peer_id));
        }
    }

    async fn handle_message(
        &mut self,
        msg: RawMessage,
        _framed_write: &mut FramedWrite<impl AsyncWrite, MsgPacketCodec>,
    ) -> Result<(), ()> {
        debug!(
            "Read message: {:?}, local_id:{:?}, peer_id:{:?}",
            msg.header(),
            self.local_id.to_base58(),
            self.peer_id.to_base58()
        );
        match msg.header().code {
            P2PMsgCode::Handshake => {
                let result = self
                    .server
                    .send(ServerEvent::Connected(
                        self.peer_id,
                        self.bound_type,
                        self.write_tx.clone(),
                        msg,
                    ))
                    .await;
                match result {
                    Ok(Ok(peer_id)) => {
                        self.handshaked = true;
                        self.peer_id = peer_id;
                    }
                    _ => return Err(()),
                }
            }
            P2PMsgCode::Block | P2PMsgCode::Consensus | P2PMsgCode::Sync => {
                self.server.try_send(ServerEvent::Message(self.peer_id, msg));
            }
            P2PMsgCode::Ping => {
                self.server.try_send(ServerEvent::Ping(self.peer_id));
            }
            _ => return Err(()),
        }
        Ok(())
    }
}

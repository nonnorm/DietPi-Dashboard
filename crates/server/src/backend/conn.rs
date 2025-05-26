use std::{collections::VecDeque, net::IpAddr};

use anyhow::{Context, Result, anyhow};
use config::PROTOCOL_VERSION;
use log::{error, info, warn};
use proto::{
    DashboardSocket,
    backend::{ActionBackendMessage, BackendMessage, Handshake, ResponseBackendMessage},
    frontend::{ActionFrontendMessage, FrontendMessage, RequestFrontendMessage},
};
use slab::Slab;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use super::{SharedBackendRegistry, cache::BackendCache};

#[derive(Debug)]
pub struct BackendInfo {
    pub nickname: String,
    pub handle: BackendHandle,
}

#[derive(Debug)]
enum BackendRequest {
    Req {
        req: RequestFrontendMessage,
        resp_tx: oneshot::Sender<ResponseBackendMessage>,
    },
    Action {
        msg: ActionFrontendMessage,
    },
    PushTerminalHandle {
        term_tx: mpsc::UnboundedSender<Vec<u8>>,
    },
}

pub struct BackendConnection {
    socket: DashboardSocket,
    registry: SharedBackendRegistry,
    addr: IpAddr,
}

impl BackendConnection {
    pub fn new(
        stream: TcpStream,
        registry: SharedBackendRegistry,
        addr: IpAddr,
        key: [u8; 32],
    ) -> Self {
        Self {
            socket: DashboardSocket::new(stream, key),
            registry,
            addr,
        }
    }

    pub async fn handle_connection(mut self) {
        let (tx, rx) = mpsc::unbounded_channel();

        let handshake = match self.read_handshake().await {
            Ok(handshake) => handshake,
            Err(err) => {
                error!("Handshake with backend {} failed: {err:#}", self.addr);
                return;
            }
        };

        if handshake.version != PROTOCOL_VERSION {
            warn!("Backend with incompatable version connected");
            return;
        }

        let nickname = if !handshake.nickname.is_empty() {
            handshake.nickname
        } else {
            self.addr.to_string()
        };

        let conn_info = BackendInfo {
            nickname,
            handle: BackendHandle::new(tx),
        };

        self.registry.lock().unwrap().insert(self.addr, conn_info);

        if let Err(err) = self.handle_requests(rx).await {
            error!("Error handling requests for backend {}: {err:#}", self.addr)
        }

        self.registry.lock().unwrap().remove(&self.addr);
    }

    async fn read_frame(&mut self) -> Result<Option<BackendMessage>> {
        self.socket
            .read_frame()
            .await
            .context("failed to read frame from socket")
    }

    async fn read_handshake(&mut self) -> Result<Handshake> {
        let message = self
            .read_frame()
            .await
            .and_then(|opt| opt.context("peer disconnected before sending handshake"))?;
        let BackendMessage::Action(ActionBackendMessage::Handshake(handshake)) = message else {
            return Err(anyhow!("peer sent invalid message, expected handshake"));
        };

        Ok(handshake)
    }

    async fn handle_requests(
        &mut self,
        mut rx: mpsc::UnboundedReceiver<BackendRequest>,
    ) -> Result<()> {
        let mut in_progress: Slab<oneshot::Sender<ResponseBackendMessage>> = Slab::new();
        let mut term_txs = Vec::new();
        let mut term_buf = VecDeque::with_capacity(10_000);
        let mut cache = BackendCache::new();

        loop {
            tokio::select! {
                chan_result = rx.recv() => {
                    let Some(conn_req) = chan_result else {
                        break;
                    };

                    match conn_req {
                        BackendRequest::Req {req, resp_tx} => {
                            if let Some(data) = cache.get(&req) {
                                let _ = resp_tx.send(data);
                                continue;
                            }

                            // Save response channel so we can send to it when we receive a response
                            let id = in_progress.insert(resp_tx) as u16;

                            let msg = FrontendMessage::Request(id, req);

                            self.socket
                                .write_frame(msg)
                                .await
                                .context("failed to write request frame")?;
                        },
                        BackendRequest::Action { msg } => {
                            let msg = FrontendMessage::Action(msg);

                            self.socket
                                .write_frame(msg)
                                .await
                                .context("failed to write action frame")?;
                        },
                        BackendRequest::PushTerminalHandle { term_tx } => {
                            if term_tx.send(term_buf.make_contiguous().to_vec()).is_ok() {
                                term_txs.push(term_tx);
                            }
                        },
                    }
                }
                resp_result = self.read_frame() => {
                    let Some(resp) = resp_result? else {
                        info!("Backend {} disconnected", self.addr);
                        break;
                    };

                    match resp {
                        BackendMessage::Response(id, data) => {
                            let Some(resp_tx) = in_progress.try_remove(id as usize) else {
                                warn!("Received frame with unknown id {} from {}", id, self.addr);
                                continue;
                            };

                            cache.insert(data.clone());

                            let _ = resp_tx.send(data);
                        },
                        BackendMessage::Action(msg) => {
                            match msg {
                                ActionBackendMessage::Handshake(_) => {
                                    warn!("Received extraneous handshake from backend {}", self.addr);
                                    continue;
                                },
                                ActionBackendMessage::Terminal(data) => {
                                    for &x in &data {
                                        if term_buf.len() == term_buf.capacity() {
                                            term_buf.pop_front();
                                        }
                                        term_buf.push_back(x);
                                    }

                                    term_txs.retain(|tx| tx.send(data.clone()).is_ok());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BackendHandle {
    tx: mpsc::UnboundedSender<BackendRequest>,
}

impl BackendHandle {
    fn new(tx: mpsc::UnboundedSender<BackendRequest>) -> Self {
        Self { tx }
    }

    pub async fn send_req(&self, req: RequestFrontendMessage) -> Result<ResponseBackendMessage> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let req = BackendRequest::Req { req, resp_tx };

        self.tx
            .send(req)
            .context("failed to send request, connection likely closed")?;

        let resp = resp_rx
            .await
            .context("failed to recv response, connection likely closed")?;

        Ok(resp)
    }

    pub async fn send_action(&self, msg: ActionFrontendMessage) -> Result<()> {
        let msg = BackendRequest::Action { msg };

        self.tx
            .send(msg)
            .context("failed to send message, connection likely closed")
    }

    pub async fn get_terminal_handle(&self) -> Result<mpsc::UnboundedReceiver<Vec<u8>>> {
        let (term_tx, term_rx) = mpsc::unbounded_channel();

        let msg = BackendRequest::PushTerminalHandle { term_tx };

        self.tx
            .send(msg)
            .context("failed to get terminal handle, connection likely closed")?;

        Ok(term_rx)
    }
}

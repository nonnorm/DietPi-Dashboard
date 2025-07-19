use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use config::PROTOCOL_VERSION;
use proto::{
    DashboardSocket,
    backend::{ActionBackendMessage, BackendMessage, Handshake, ResponseBackendMessage},
    frontend::{ActionFrontendMessage, FrontendMessage, RequestFrontendMessage},
};
use sysinfo::{Components, Disks, Networks, System};
use tokio::{net::TcpStream, sync::mpsc};

use crate::{SharedConfig, actions, getters};

macro_rules! getters {
    ($req:expr, $ctx:expr, {
        $( $variant:ident $(($data:ident))? => $fn:expr, )*
    }) => {
        match $req {
            $( RequestFrontendMessage::$variant $(($data))? => {
                let data = tokio::task::spawn_blocking(move || $fn($ctx $(, $data)?)).await.unwrap();
                ResponseBackendMessage::$variant(data)
            } )*
        }
    };
}

pub type SharedSystem = Arc<Mutex<SystemComponents>>;

pub struct SystemComponents {
    pub system: System,
    pub components: Components,
    pub disks: Disks,
    pub networks: Networks,
}

impl SystemComponents {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            components: Components::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
        }
    }
}

#[derive(Clone)]
pub struct BackendContext {
    pub config: SharedConfig,
    pub system: SharedSystem,
    pub socket_tx: mpsc::UnboundedSender<BackendMessage>,
    pub term_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl BackendContext {
    pub fn system(&mut self) -> impl std::ops::DerefMut<Target = SystemComponents> {
        self.system.lock().unwrap()
    }
}

pub struct BackendClient {
    socket: DashboardSocket,
    context: BackendContext,
    rx: mpsc::UnboundedReceiver<BackendMessage>,
}

impl BackendClient {
    pub async fn new(
        context: BackendContext,
        rx: mpsc::UnboundedReceiver<BackendMessage>,
    ) -> Result<Self> {
        let stream = TcpStream::connect(context.config.frontend_addr)
            .await
            .context("failed to connect to frontend")?;

        Ok(Self {
            socket: DashboardSocket::new(stream, context.config.secret.0),
            context,
            rx,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        self.send_handshake().await?;

        loop {
            tokio::select! {
                frame_result = self.socket.read_frame() => {
                    let req: FrontendMessage = frame_result
                        .context("failed to read frame from frontend")?
                        .context("frontend unexpectedly disconnected")?;

                    let handler = RequestHandler::new(req, self.context.clone());
                    tokio::spawn(handler.run());
                }
                chan_result = self.rx.recv() => {
                    // Since we hold a copy of the sender, it should be impossible for this to return None
                    let frame = chan_result.unwrap();

                    self.socket.write_frame(frame).await.context("failed to send response")?;
                }
            }
        }
    }

    async fn send_handshake(&mut self) -> Result<()> {
        let nickname = self.context.config.nickname.clone();

        let handshake = Handshake {
            nickname,
            version: PROTOCOL_VERSION,
        };

        let msg = ActionBackendMessage::Handshake(handshake);
        let msg = BackendMessage::Action(msg);

        self.socket
            .write_frame(msg)
            .await
            .context("failed to send handshake")
    }
}

struct RequestHandler {
    req: FrontendMessage,
    context: BackendContext,
}

impl RequestHandler {
    fn new(req: FrontendMessage, context: BackendContext) -> Self {
        Self { req, context }
    }

    async fn run(self) {
        let ctx = self.context.clone();

        match self.req {
            FrontendMessage::Request(id, req) => {
                let resp = getters!(req, ctx, {
                    Cpu => getters::cpu,
                    Temp => getters::temp,
                    Mem => getters::memory,
                    Disk => getters::disks,
                    NetIO => getters::network_io,
                    Processes => getters::processes,
                    Host => getters::host,
                    Software => getters::software,
                    Command(action) => getters::command,
                    Services => getters::services,
                    Directory(path) => getters::list_directory,
                    Download(path) => getters::read_file,
                });

                let resp = BackendMessage::Response(id, resp);
                let _ = self.context.socket_tx.send(resp);
            }
            FrontendMessage::Action(msg) => match msg {
                ActionFrontendMessage::Terminal(data) => {
                    let _ = self.context.term_tx.send(data);
                }
                ActionFrontendMessage::Signal(action) => {
                    tokio::task::spawn_blocking(|| actions::process_signal(ctx, action))
                        .await
                        .unwrap()
                }
                ActionFrontendMessage::NewFile(path) => actions::new_file(path).await,
                ActionFrontendMessage::NewFolder(path) => actions::new_folder(path).await,
                ActionFrontendMessage::Rename(action) => actions::rename(action).await,
                ActionFrontendMessage::DeleteFile(path) => actions::delete_file(path).await,
                ActionFrontendMessage::DeleteFolder(path) => actions::delete_folder(path).await,
            },
        }
    }
}

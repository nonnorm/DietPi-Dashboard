use anyhow::{Context, Result};
use proto::backend::{ActionBackendMessage, BackendMessage};
use pty_process::{Command, Pty, Size};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};

fn create_pty() -> Result<Pty> {
    let (pty, pts) = pty_process::open().context("failed to open pty")?;
    pty.resize(Size::new(24, 80))
        .context("failed to resize pty")?;

    let cmd = Command::new("login");
    cmd.spawn(pts).context("failed to spawn terminal")?;

    Ok(pty)
}

pub struct Terminal {
    socket_tx: mpsc::UnboundedSender<BackendMessage>,
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    pty: Pty,
}

impl Terminal {
    pub fn new(
        socket_tx: mpsc::UnboundedSender<BackendMessage>,
        rx: mpsc::UnboundedReceiver<Vec<u8>>,
    ) -> Result<Self> {
        let pty = create_pty()?;

        Ok(Self { socket_tx, rx, pty })
    }

    pub async fn run(mut self) {
        let mut buf = [0; 512];

        loop {
            loop {
                tokio::select! {
                    data = self.rx.recv() => {
                        let Some(data): Option<Vec<u8>> = data else {
                            break;
                        };

                        if self.pty.write_all(&data).await.is_err() {
                            break;
                        }
                    }
                    n = self.pty.read(&mut buf) => {
                        let Ok(n) = n else {
                            break;
                        };

                        if n == 0 {
                            break;
                        }

                        let msg = ActionBackendMessage::Terminal(buf[..n].to_vec());
                        let msg = BackendMessage::Action(msg);

                        let _ = self.socket_tx.send(msg);
                    }
                }
            }

            if let Ok(pty) = create_pty() {
                self.pty = pty;
            } else {
                break;
            }
        }
    }
}

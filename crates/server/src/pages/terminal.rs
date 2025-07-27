use maud::html;

use futures_util::{SinkExt, StreamExt};
use proto::frontend::ActionFrontendMessage;
use tokio_tungstenite::tungstenite::Message;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::template;

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let content = html! {
        section {
            h2 { "Terminal" }
            web-terminal {}
        }
    };

    template(&req, content)
}

pub async fn socket(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let backend = req.extract_backends()?.current_backend.handle;

    req.extract_websocket(async move |mut ws| {
        let mut term_rx = backend.get_terminal_handle().await.unwrap();

        loop {
            tokio::select! {
                data = term_rx.recv() => {
                    let Some(data) = data else {
                        break;
                    };

                    if ws.send(Message::binary(data)).await.is_err() {
                        break;
                    }
                }
                data = ws.next() => {
                    let Some(Ok(data)) = data else {
                        break;
                    };
                    let data = data.into_data().to_vec();

                    let msg = ActionFrontendMessage::Terminal(data);

                    if backend.send_action(msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    })
}

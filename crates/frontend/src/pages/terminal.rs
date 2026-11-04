use futures_util::StreamExt;
use http_body_util::BodyDataStream;
use hyper::{StatusCode, header};
use maud::html;
use proto::frontend::{ActionFrontendMessage, TerminalDimensions};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::template;

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let content = html! {
        section {
            h2 data-i18n="terminal_title" { "Terminal" }
            web-terminal {}
        }
    };

    template(&req, content, "")
}

pub async fn stream(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let backend = req.extract_backends()?.current_backend.handle;
    let term_rx = backend.get_terminal_handle().await.unwrap();

    let term_stream = UnboundedReceiverStream::new(term_rx);

    Ok(ServerResponse::new()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .stream_body(term_stream))
}

// Someday we'll be able to stream it in? Right now this is a handwritten version of http_body_util::Collected.
pub async fn write(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;
    let backend = req.extract_backends()?.current_backend.handle;

    let body = req.extract_body().await?;
    let mut body = BodyDataStream::new(body);

    while let Some(Ok(data)) = body.next().await {
        let msg = ActionFrontendMessage::Terminal(data.to_vec());
        if backend.send_action(msg).await.is_err() {
            return Err(ServerResponse::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("failed to write to backend processor to write to terminal"));
        }
    }

    Ok(ServerResponse::new())
}

pub async fn resize(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;
    let backend = req.extract_backends()?.current_backend.handle;

    let size: TerminalDimensions = req.extract_form().await?;

    let msg = ActionFrontendMessage::ResizeTerminal(size);
    if backend.send_action(msg).await.is_err() {
        Err(ServerResponse::new()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("failed to write to backend processor to resize terminal"))
    } else {
        Ok(ServerResponse::new())
    }
}

use std::{
    collections::HashMap,
    net::IpAddr,
    ops::{Deref, DerefMut},
};

use config::frontend::FrontendConfig;
use data_encoding::BASE64;
use http_body_util::BodyExt;
use hyper::{
    StatusCode,
    body::Incoming,
    header,
    http::request::Parts as RequestParts,
    upgrade::{self, Upgraded},
};
use hyper_util::rt::TokioIo;
use proto::{
    backend::ResponseBackendMessage,
    frontend::{ActionFrontendMessage, RequestFrontendMessage},
};
use ring::digest::SHA1_FOR_LEGACY_USE_ONLY;
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Role};

use crate::backend::BackendHandle;

use super::{
    FrontendContext,
    auth::SharedLoginMap,
    response::{RedirectType, ServerResponse},
};

pub type HyperRequest = hyper::Request<Incoming>;

fn get_cookies(parts: &RequestParts) -> HashMap<String, String> {
    let cookie_header = parts
        .headers
        .get(header::COOKIE)
        .and_then(|x| x.to_str().ok())
        .unwrap_or_default();

    cookie_header
        .split("; ")
        .filter_map(|x| x.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub struct BackendData {
    pub backend_list: Vec<(IpAddr, String)>,
    pub current_backend: (IpAddr, BackendHandle),
}

pub struct ServerRequest {
    parts: RequestParts,
    body: Option<Incoming>,
    cookies: HashMap<String, String>,
    context: FrontendContext,
}

impl ServerRequest {
    pub fn new(req: HyperRequest, context: FrontendContext) -> Self {
        let (parts, body) = req.into_parts();

        let cookies = get_cookies(&parts);

        Self {
            parts,
            body: Some(body),
            cookies,
            context,
        }
    }

    pub fn path_segments(&self) -> impl Iterator<Item = &str> {
        self.uri.path().split('/').filter(|x| !x.is_empty())
    }

    pub fn config(&self) -> &FrontendConfig {
        &self.context.config
    }

    pub fn extract_backends(&self) -> Result<BackendData, ServerResponse> {
        let backends = self.context.backends.lock().unwrap();
        let backend_list: Vec<_> = backends
            .iter()
            .map(|(addr, info)| (*addr, info.nickname.clone()))
            .collect();

        if backend_list.is_empty() {
            return Err(ServerResponse::new()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body("no connected backends"));
        }

        let current_backend = {
            let cookie_ip = self
                .cookies
                .get("backend")
                .and_then(|x| x.parse::<IpAddr>().ok());

            let (addr, backend_info) = cookie_ip
                .and_then(|x| backends.get_key_value(&x))
                .or_else(|| backends.get_key_value(&backend_list[0].0))
                .unwrap();

            (*addr, backend_info.handle.clone())
        };

        Ok(BackendData {
            backend_list,
            current_backend,
        })
    }

    pub async fn send_backend_req(
        &self,
        req: RequestFrontendMessage,
    ) -> Result<ResponseBackendMessage, ServerResponse> {
        let backend_handle = self.extract_backends()?.current_backend.1;

        backend_handle.send_req(req).await.map_err(|err| {
            ServerResponse::new()
                .status(StatusCode::BAD_GATEWAY)
                .body(format!("backend request failed: {err}"))
        })
    }

    pub async fn send_backend_action(
        &self,
        msg: ActionFrontendMessage,
    ) -> Result<(), ServerResponse> {
        let backend_handle = self.extract_backends()?.current_backend.1;

        backend_handle.send_action(msg).await.map_err(|err| {
            ServerResponse::new()
                .status(StatusCode::BAD_GATEWAY)
                .body(format!("backend action failed: {err}"))
        })
    }

    pub fn extract_query<Qu: serde::de::DeserializeOwned>(&self) -> Result<Qu, ServerResponse> {
        let query = self.uri.query().unwrap_or_default();

        serde_urlencoded::from_str(query).map_err(|err| {
            ServerResponse::new()
                .status(StatusCode::BAD_REQUEST)
                .body(format!("invalid query params: {err}"))
        })
    }

    pub async fn extract_form<T: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<T, ServerResponse> {
        let Some(body) = self.body.take() else {
            return Err(ServerResponse::new()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("form already extracted"));
        };

        let body = body.collect().await.map_err(|_| {
            ServerResponse::new()
                .status(StatusCode::BAD_REQUEST)
                .body("needs body")
        })?;

        serde_urlencoded::from_bytes(&body.to_bytes()).map_err(|_| {
            ServerResponse::new()
                .status(StatusCode::BAD_REQUEST)
                .body("invalid form body")
        })
    }

    pub fn is_fixi(&self) -> bool {
        self.headers.contains_key("nm-request")
    }

    pub fn extract_websocket<F, Fut>(self, handler_fn: F) -> Result<ServerResponse, ServerResponse>
    where
        F: FnOnce(WebSocketStream<TokioIo<Upgraded>>) -> Fut + Send + 'static,
        Fut: Future + Send,
    {
        const WEBSOCKET_MAGIC_NUM: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

        let check_header = |k, v: &[u8]| {
            self.headers
                .get(k)
                .map(|x| x.as_bytes())
                .is_some_and(|x| x.windows(v.len()).any(|subslice| subslice == v))
        };

        let is_websocket_req = check_header(header::CONNECTION, b"Upgrade")
            && check_header(header::UPGRADE, b"websocket")
            && check_header(header::SEC_WEBSOCKET_VERSION, b"13")
            && self.headers.contains_key(header::SEC_WEBSOCKET_KEY);

        if !is_websocket_req {
            return Err(ServerResponse::new()
                .status(StatusCode::BAD_REQUEST)
                .body("expected websocket upgrade"));
        }

        let sec_key = self
            .headers
            .get(header::SEC_WEBSOCKET_KEY)
            .unwrap()
            .as_bytes();

        let resp_key = ring::digest::digest(
            &SHA1_FOR_LEGACY_USE_ONLY,
            &[sec_key, WEBSOCKET_MAGIC_NUM].concat(),
        );
        let resp_key = BASE64.encode(resp_key.as_ref());

        let resp = ServerResponse::new()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(header::CONNECTION, "Upgrade")
            .header(header::UPGRADE, "websocket")
            .header(header::SEC_WEBSOCKET_ACCEPT, resp_key);

        let req = hyper::Request::from_parts(self.parts, self.body.unwrap());

        tokio::spawn(async {
            if let Ok(stream) = upgrade::on(req).await {
                let stream = TokioIo::new(stream);
                let ws = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;
                handler_fn(ws).await;
            }
        });

        Ok(resp)
    }

    pub fn check_login(&self) -> Result<(), ServerResponse> {
        if self.config().enable_login {
            let err_resp = if self.is_fixi() {
                Err(ServerResponse::new()
                    .body(r#"<meta http-equiv="refresh" content="0; url=/login" />"#))
            } else {
                Err(ServerResponse::new().redirect(RedirectType::SeeOther, "/login"))
            };

            let Some(token) = self.cookies.get("token") else {
                return err_resp;
            };

            if !self.context.logins.get().contains_token(token) {
                return err_resp;
            }
        }

        Ok(())
    }

    pub fn extract_logins(&self) -> SharedLoginMap {
        self.context.logins.clone()
    }
}

impl Deref for ServerRequest {
    type Target = RequestParts;

    fn deref(&self) -> &Self::Target {
        &self.parts
    }
}

impl DerefMut for ServerRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parts
    }
}

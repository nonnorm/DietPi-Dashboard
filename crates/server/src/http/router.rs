use hyper::{Method, StatusCode};

use crate::pages::*;

use super::response::{BuiltResponse, RedirectType, ServerResponse};
use super::{request::ServerRequest, statics};

const GET: &Method = &Method::GET;
const POST: &Method = &Method::POST;

macro_rules! router {
    ($req:expr, $path:expr, {
        $( ($method:pat, $paths:pat) => $handler:expr, )*
        _ => $fallback:expr,
    }) => {{
        match (&$req.method, $path) {
            $(
                ($method, $paths) => match $handler($req).await {
                    Ok(resp) | Err(resp) => resp
                },
            )*
            _ => $fallback()
        }
    }};
}

pub async fn router(req: ServerRequest) -> Result<BuiltResponse, std::convert::Infallible> {
    let path_segments: Vec<_> = req.path_segments().collect();

    let resp = router!(req, &*path_segments, {
        (GET, ["static", "main.css"]) => statics::css,
        (GET, ["static", "main.js"]) => statics::js,
        (GET, ["static", "icons.svg"]) => statics::icons,

        (GET, []) => async |_| { Ok(ServerResponse::new().redirect(RedirectType::Permanent, "/system")) },

        (GET, ["login"]) => login::page,
        (POST, ["login"]) => login::form,

        (GET, ["system"]) => system::page,

        (GET, ["process"]) => process::page,
        (GET, ["process", "signal"]) => process::signal,

        (GET, ["software"]) => software::page,
        (POST, ["software"]) => software::form,

        (GET, ["service"]) => service::page,

        (GET, ["management"]) => management::page,

        (GET, ["terminal"]) => terminal::page,
        (GET, ["terminal", "ws"]) => terminal::socket,

        _ => || { ServerResponse::new().status(StatusCode::NOT_FOUND).body("page not found") },
    });
    let resp = resp.build();

    Ok(resp)
}

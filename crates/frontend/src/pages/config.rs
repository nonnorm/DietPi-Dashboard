use hyper::StatusCode;
use maud::html;
use tokio::fs;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

async fn read_config() -> Result<String, ServerResponse> {
    let mut cfgpath = std::env::current_exe().map_err(|_| {
        ServerResponse::new()
            .body("failed to get config path")
            .status(StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    cfgpath.set_file_name("config-frontend.toml");

    fs::read_to_string(cfgpath).await.map_err(|_| {
        ServerResponse::new()
            .body("failed to read config")
            .status(StatusCode::INTERNAL_SERVER_ERROR)
    })
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let frontend_cfg = read_config().await?;
    let backend_cfg = send_req!(req, ReadConfig)?;

    let content = html! {
        section {
            h2 { "Frontend Config" }

            pre {
                (frontend_cfg)
            }
        }
        br;
        section {
            h2 { "Backend Config" }

            pre {
                (backend_cfg)
            }
        }
    };

    template(&req, content)
}

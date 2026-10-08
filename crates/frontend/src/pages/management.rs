use std::time::Duration;

use maud::html;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

use hyper::StatusCode;
use tokio::fs;

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

    let data = send_req!(req, Host)?;
    let frontend_cfg = read_config().await?;
    let backend_cfg = send_req!(req, ReadConfig)?;

    let pretty_time = humantime::format_duration(Duration::from_secs(data.uptime));

    let content = html! {
        section {
            h2 data-i18n="host_information" { "Host Information" }

            table .management-table {
                tr {
                    td data-i18n="hostname" { "Hostname" }
                    td { (data.hostname) }
                }
                tr {
                    td data-i18n="network_interface" { "Network Interface" }
                    td { (data.nic) }
                }
                tr {
                    td data-i18n="uptime" { "Uptime" }
                    td { (pretty_time) }
                }
                tr {
                    td data-i18n="installed_packages" { "Installed Packages" }
                    td { (data.num_pkgs) }
                }
                tr {
                    td data-i18n="os_version" { "OS Version" }
                    td { (data.os_version) }
                }
                tr {
                    td data-i18n="kernel_version" { "Kernel Version" }
                    td { (data.kernel) }
                }
                tr {
                    td data-i18n="dietpi_version" { "DietPi Version" }
                    td { (data.dp_version) }
                }
                tr {
                    td data-i18n="architecture" { "Architecture" }
                    td { (data.arch) }
                }
            }
        }
        br;
        section {
            h2 data-i18n="frontend_config" { "Frontend Config" }

            pre {
                (frontend_cfg)
            }
        }
        br;
        section {
            h2 data-i18n="backend_config" { "Backend Config" }

            pre {
                (backend_cfg)
            }
        }
        @if req.config().enable_login {
            br;
            section {
                h2 data-i18n="dashboard_administration" { "Dashboard Administration" }

                form action="/logout" method="POST" {
                    button .logout data-i18n="logout" { "Logout" }
                }
            }
        }
    };

    template(&req, content, "")
}

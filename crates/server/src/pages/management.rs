use std::time::Duration;

use maud::html;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let data = send_req!(req, Host)?;

    let pretty_time = humantime::format_duration(Duration::from_secs(data.uptime));

    let content = html! {
        section {
            h2 { "Host Information" }

            table .management-table {
                tr {
                    td { "Hostname" }
                    td { (data.hostname) }
                }
                tr {
                    td { "Network Interface" }
                    td { (data.nic) }
                }
                tr {
                    td { "Uptime" }
                    td { (pretty_time) }
                }
                tr {
                    td { "Installed Packages" }
                    td { (data.num_pkgs) }
                }
                tr {
                    td { "OS Version" }
                    td { (data.os_version) }
                }
                tr {
                    td { "Kernel Version" }
                    td { (data.kernel) }
                }
                tr {
                    td { "DietPi Version" }
                    td { (data.dp_version) }
                }
                tr {
                    td { "Architecture" }
                    td { (data.arch) }
                }
            }
        }
    };

    template(&req, content)
}

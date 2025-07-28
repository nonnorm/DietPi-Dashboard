use maud::html;
use proto::backend::ServiceStatus;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let data = send_req!(req, Services)?;

    let content = html! {
        section {
            h2 { "Services" }
            table {
                tr {
                    th { "Name" }
                    th { "Status" }
                    th { "Error Log" }
                    th { "Start Time" }
                }
                @for service in data.services {
                    tr {
                        td { (service.name) }
                        td {
                            @match service.status {
                                ServiceStatus::Active => "active",
                                ServiceStatus::Inactive => "inactive",
                                ServiceStatus::Failed => "failed",
                                ServiceStatus::Unknown => "unknown"
                            }
                        }
                        td {
                            @if !service.err_log.is_empty() {
                                details {
                                    summary { "View log" }
                                    pre {
                                        (service.err_log)
                                    }
                                }
                            }
                        }
                        td { (service.start) }
                    }
                }
            }
        }
    };

    template(&req, content)
}

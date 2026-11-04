use maud::html;
use proto::backend::ServiceStatus;

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let data = send_req!(req, Services)?;

    let content = html! {
        section {
            h2 data-i18n="services_title" { "Services" }
            table {
                tr {
                    th data-i18n="name" { "Name" }
                    th data-i18n="status" { "Status" }
                    th data-i18n="error_log" { "Error Log" }
                    th data-i18n="start_time" { "Start Time" }
                }
                @for service in data.services {
                    @let (status_attr, status_label, status_i18n) = match service.status {
                        ServiceStatus::Active => ("active", "Active", "active"),
                        ServiceStatus::Inactive => ("inactive", "Inactive", "inactive"),
                        ServiceStatus::Failed => ("failed", "Failed", "failed"),
                        ServiceStatus::Unknown => ("unknown", "Unknown", "unknown"),
                    };
                    tr {
                        td { (service.name) }
                        td {
                            span .status-badge data-status=(status_attr) data-i18n=(status_i18n) { (status_label) }
                        }
                        td {
                            @if !service.err_log.is_empty() {
                                details {
                                    summary data-i18n="view_log" { "View log" }
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

    template(&req, content, "")
}

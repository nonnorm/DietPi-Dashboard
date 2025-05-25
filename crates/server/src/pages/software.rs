use maud::{Markup, html};
use proto::{backend::SoftwareInfo, frontend::CommandAction};
use serde::Deserialize;

use crate::http::{query_array::QueryArray, request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

fn software_table(list: &[SoftwareInfo], action: &str) -> Markup {
    html! {
        server-swap trigger="submit" target="#output" method="POST" disable={"input[value='" (action) "']"} {
            array-form array-name="software" {
                form {
                    table {
                        tr {
                            th { "Name" }
                            th { "Description" }
                            th { "Dependencies" }
                            th { "Docs" }
                            th { (action) }
                        }
                        @for item in list {
                            tr {
                                td { (item.name) }
                                td { (item.desc) }
                                td { (item.deps) }
                                td {
                                    @if item.docs.starts_with("http") {
                                        a href=(item.docs) { (item.docs) }
                                    } @else {
                                        (item.docs)
                                    }
                                }
                                td {
                                    input type="checkbox" name="software" value=(item.id);
                                }
                            }
                        }
                    }
                    br;
                    input .software-input type="submit" name="action" value=(action);
                }
            }
        }
    }
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let data = send_req!(req, Software)?;

    let content = html! {
        section {
            h2 { "Installed Software" }
            (software_table(&data.installed, "Uninstall"))
        }
        br;
        section {
            h2 { "Not Installed Software" }
            (software_table(&data.uninstalled, "Install"))
        }
        br;
        #output {}
    };

    template(&req, content)
}

#[derive(Deserialize)]
enum SoftwareAction {
    Install,
    Uninstall,
}

#[derive(Deserialize)]
struct SoftwareForm {
    software: QueryArray,
    action: SoftwareAction,
}

pub async fn form(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let form: SoftwareForm = req.extract_form().await?;

    let action = match form.action {
        SoftwareAction::Install => "install",
        SoftwareAction::Uninstall => "uninstall",
    }
    .into();

    let mut args = vec![action];
    for id in form.software.iter::<u16>() {
        args.push(id.to_string());
    }

    let msg = CommandAction {
        cmd: "/boot/dietpi/dietpi-software".into(),
        args,
    };

    let resp = send_req!(req, Command(msg))?;
    let output = String::from_utf8_lossy(&resp.output);

    let content = html! {
        section #output {
            h2 { "Install Summary" }
            pre {
                (output)
            }
        }
    };

    template(&req, content)
}

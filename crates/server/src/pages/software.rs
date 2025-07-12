use maud::{Markup, html};
use proto::{backend::SoftwareInfo, frontend::CommandAction};
use serde::Deserialize;

use crate::{
    http::{query_array::QueryArray, request::ServerRequest, response::ServerResponse},
    pages::template::Icon,
};

use super::template::{send_req, template};

fn software_table(list: &[SoftwareInfo], pretty_action: &str, action: &str) -> Markup {
    html! {
        div nm-data="software: new Map()" {
            table {
                tr {
                    th { "Name" }
                    th { "Description" }
                    th { "Dependencies" }
                    th { "Docs" }
                    th { (pretty_action) }
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
                            input type="checkbox" nm-bind={
                                "onchange: () => {
                                    this.checked ? software.set("(item.id)", '"(item.name)"') : software.delete("(item.id)");
                                    software = software;
                                }"
                            };
                        }
                    }
                }
            }
            br;
            button .software-input
                value=(action)
                nm-bind="
                    onclick: () => post('/software', { software: Array.from(software.keys()).join(','), action: this.value }),
                    disabled: () => nmFetching
                "
             {
                span .spinner { (Icon::new("svg-spinners-180-ring")) }
                (pretty_action) " "
                span nm-bind="textContent: () => Array.from(software.values()).join(', ')" {}
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
            (software_table(&data.installed, "Uninstall", "uninstall"))
        }
        br;
        section {
            h2 { "Not Installed Software" }
            (software_table(&data.uninstalled, "Install", "install"))
        }
        br;
        #output {}
    };

    template(&req, content)
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
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
        section #output nm-bind="_: this.scrollIntoView" {
            h2 { "Install Summary" }
            pre {
                (output)
            }
        }
    };

    template(&req, content)
}

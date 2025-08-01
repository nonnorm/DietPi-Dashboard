use maud::{Markup, html};
use proto::{
    backend::{SoftwareInfo, SoftwareResponse},
    frontend::CommandAction,
};
use serde::Deserialize;

use crate::{
    http::{query_array::QueryArray, request::ServerRequest, response::ServerResponse},
    pages::template::Icon,
};

use super::template::{send_req, template};

fn software_table(list: &[SoftwareInfo], idx: u8, pretty_action: &str, action: &str) -> Markup {
    html! {
        div nm-bind={"hidden: () => activeIdx !== " (idx)} {
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
                        onclick: () => post('/software', { software: [...software.keys()].join(','), action: this.value }),
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
}

fn card(data: &SoftwareResponse) -> Markup {
    html! {
        section #software-card {
            h2 { "Software" }
            .tab-container role="tablist" nm-data="activeIdx: 0" {
                .tabs {
                    button nm-bind="
                        onclick: () => activeIdx = 0,
                        ariaSelected: () => activeIdx === 0
                    " { "Install Software" }
                    button nm-bind="
                        onclick: () => activeIdx = 1,
                        ariaSelected: () => activeIdx === 1
                    " { "Uninstall Software" }
                }
                (software_table(&data.uninstalled, 0, "Install", "install"))
                (software_table(&data.installed, 1, "Uninstall", "uninstall"))
            }
        }
    }
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let data = send_req!(req, Software)?;

    let content = html! {
        (card(&data))
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

    let data = send_req!(req, Software)?;

    let content = html! {
        (card(&data))
        br;
        section #output nm-bind="_: () => this.scrollIntoView()" {
            h2 { "Install Summary" }
            pre {
                (output)
            }
        }
    };

    template(&req, content)
}

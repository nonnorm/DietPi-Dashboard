use maud::{Markup, html};
use pretty_bytes_typed::pretty_bytes;
use proto::backend::FileKind;
use serde::Deserialize;

use crate::{
    http::{
        request::ServerRequest,
        response::{RedirectType, ServerResponse},
    },
    pages::template::{Icon, send_act},
};

use super::template::{send_req, template};

fn default_path() -> String {
    "/root".into()
}

#[derive(Deserialize)]
pub struct BrowserQuery {
    #[serde(default = "default_path")]
    path: String,
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: BrowserQuery = req.extract_query()?;

    let data = send_req!(req, Directory(query.path.clone()))?;

    let content = html! {
        #browser-swap nm-data="selectedRow: null, viewHidden: false" {
            #path-display {
                @let paths = query.path.split_inclusive('/').scan(String::new(), |acc, segment| {
                    acc.push_str(segment);
                    Some((acc.clone(), segment))
                });

                @for (full_path, path_segment) in paths {
                    button nm-bind={"onclick: () => get('/browser', {path: '"(full_path)"'})"} { (path_segment) }
                }
            }

            table #browser-table {
                tr {
                    th { "File Name" }
                    th { "File Size" }
                }
                @for item in data.dir_list {
                    @let name = item.path.rsplit_once('/').map(|(_, name)| name).unwrap_or(&item.path);
                    @let icon = match item.kind {
                        FileKind::TextFile => "fa6-solid-file-lines",
                        FileKind::BinaryFile => "fa6-solid-file",
                        FileKind::Directory => "fa6-solid-folder",
                        FileKind::Special => "fa6-solid-cube",
                    };
                    @let pretty_size = item.size.map(|size| pretty_bytes(size, Some(0)).to_string()).unwrap_or_else(|| "--".into());
                    tr data-hidden[name.starts_with('.')] nm-bind={"
                        ariaCurrent: () => selectedRow === this,
                        hidden: () => this.dataset.hidden !== undefined && !viewHidden,
                        onclick: () => {
                            selectedRow = this;
                            get('/browser/actions', {
                                current_path: '"(query.path)"',
                                path: '"(item.path)"',
                                kind: '"(serde_plain::to_string(&item.kind).unwrap())"'
                            })
                        },
                        ondblclick: () => {
                            get('/browser', {path: '"(item.path)"'});
                        }
                    "} {
                        td {
                            (Icon::new(icon).size(18)) " " (name)
                        }
                        td { (pretty_size) }
                    }
                }
            }
            #actions-list {
                (default_actions(&query.path))
            }
        }
    };

    template(&req, content)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserActionsQuery {
    current_path: String,
    path: String,
    kind: FileKind,
}

pub async fn actions(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: BrowserActionsQuery = req.extract_query()?;

    let content = html! {
        div #actions-list {
            (default_actions(&query.current_path))
        }
    };

    template(&req, content)
}

fn default_actions(current_path: &str) -> Markup {
    html! {
        button title="Show Hidden Files" nm-bind="
            onclick: () => viewHidden = true,
            hidden: () => viewHidden
        " { (Icon::new("fa6-solid-eye-slash")) }
        button title="Hide Hidden Files" nm-bind="
            onclick: () => viewHidden = false,
            hidden: () => !viewHidden
        " { (Icon::new("fa6-solid-eye")) }
        button title="Refresh" nm-bind={ "onclick: () => get('/browser', {path: '"(current_path)"'})" } {
            (Icon::new("fa6-solid-rotate"))
        }
        button title="New File" nm-bind={"
            onclick: () => { 
                let name = prompt('Enter a file name:');
                if (name) post('/browser/actions/new-file', {parent: '"(current_path)"', name});
            }
        "} { (Icon::new("fa6-solid-file-medical")) }
        button title="New Folder" nm-bind={"
            onclick: () => { 
                let name = prompt('Enter a folder name:');
                if (name) post('/browser/actions/new-folder', {parent: '"(current_path)"', name});
            }
        "} { (Icon::new("fa6-solid-folder-plus")) }
    }
}

#[derive(Deserialize)]
pub struct NewQuery {
    parent: String,
    name: String,
}

pub async fn new_file(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: NewQuery = req.extract_form().await?;

    let path = format!("{}{}", query.parent, query.name);

    send_act!(req, NewFile(path))?;

    Ok(ServerResponse::new().redirect(
        RedirectType::SeeOther,
        &format!("/browser?path={}", query.parent),
    ))
}

pub async fn new_folder(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: NewQuery = req.extract_form().await?;

    let path = format!("{}{}", query.parent, query.name);

    send_act!(req, NewFolder(path))?;

    Ok(ServerResponse::new().redirect(
        RedirectType::SeeOther,
        &format!("/browser?path={}", query.parent),
    ))
}

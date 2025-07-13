use std::path::Path;

use hyper::StatusCode;
use maud::{Markup, html};
use pretty_bytes_typed::pretty_bytes;
use proto::{backend::FileKind, frontend::RenameAction};
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
        #browser-swap nm-data="selectedRow: null" {
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
                    tr nm-bind={"
                        ariaCurrent: () => selectedRow === this,
                        hidden: () => this.dataset.hidden !== undefined && !viewHidden,
                        onclick: () => {
                            selectedRow = this;
                            get('/browser/actions', {
                                currentPath: '"(query.path)"',
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
            @if matches!(query.kind, FileKind::TextFile | FileKind::BinaryFile | FileKind::Directory) {
                button title="Rename" nm-bind={"
                    onclick: () => {
                        let new_name = prompt('Enter a new name:');
                        if (new_name) post('/browser/actions/rename', {
                            path: '"(query.path)"',
                            new_name
                        });
                    }
                "} {
                    (Icon::new("fa6-solid-i-cursor"))
                }
            }
            @if matches!(query.kind, FileKind::TextFile | FileKind::BinaryFile)  {
                button title="Delete" nm-bind={"
                    onclick: () => { 
                        if (confirm('Are you sure you want to delete this file?'))
                            post('/browser/actions/delete-file', {path: '"(query.path)"'});
                    }
                "} { (Icon::new("fa6-solid-trash")) }
            }
            @if matches!(query.kind, FileKind::Directory)  {
                button title="Delete" nm-bind={"
                    onclick: () => { 
                        if (confirm('Are you sure you want to delete this folder?'))
                            post('/browser/actions/delete-folder', {path: '"(query.path)"'});
                    }
                "} { (Icon::new("fa6-solid-trash")) }
            }
        }
    };

    template(&req, content)
}

fn default_actions(current_path: &str) -> Markup {
    html! {
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

    // Path is guaranteed to be a valid string, because it was built from two strings
    let path = Path::new(&query.parent).join(Path::new(&query.name));
    let path = path.into_os_string().into_string().unwrap();

    send_act!(req, NewFolder(path))?;

    Ok(ServerResponse::new().redirect(
        RedirectType::SeeOther,
        &format!("/browser?path={}", query.parent),
    ))
}

#[derive(Deserialize)]
pub struct RenameQuery {
    path: String,
    new_name: String,
}

pub async fn rename(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: RenameQuery = req.extract_form().await?;

    let parent = Path::new(&query.path).parent().ok_or(
        ServerResponse::new()
            .status(StatusCode::BAD_REQUEST)
            .body("can't get parent of path"),
    )?;
    let parent = parent.as_os_str().to_str().unwrap();

    let new_path = Path::new(&query.path).with_file_name(&query.new_name);
    let new_path = new_path.into_os_string().into_string().unwrap();

    let action = RenameAction {
        from: query.path.clone(),
        to: new_path,
    };

    send_act!(req, Rename(action))?;

    Ok(ServerResponse::new().redirect(RedirectType::SeeOther, &format!("/browser?path={parent}")))
}

#[derive(Deserialize)]
pub struct FileQuery {
    path: String,
}

pub async fn delete_file(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: FileQuery = req.extract_form().await?;

    let parent = Path::new(&query.path).parent().ok_or(
        ServerResponse::new()
            .status(StatusCode::BAD_REQUEST)
            .body("can't get parent of path"),
    )?;
    let parent = parent.as_os_str().to_str().unwrap();

    send_act!(req, DeleteFile(query.path.clone()))?;

    Ok(ServerResponse::new().redirect(RedirectType::SeeOther, &format!("/browser?path={parent}")))
}

pub async fn delete_folder(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: FileQuery = req.extract_form().await?;

    let parent = Path::new(&query.path).parent().ok_or(
        ServerResponse::new()
            .status(StatusCode::BAD_REQUEST)
            .body("can't get parent of path"),
    )?;
    let parent = parent.as_os_str().to_str().unwrap();

    send_act!(req, DeleteFolder(query.path.clone()))?;

    Ok(ServerResponse::new().redirect(RedirectType::SeeOther, &format!("/browser?path={parent}")))
}

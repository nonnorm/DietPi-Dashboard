use std::path::{Path, PathBuf};

use data_encoding::BASE64;
use hyper::StatusCode;
use maud::{Markup, html};
use pretty_bytes_typed::pretty_bytes;
use proto::{
    backend::FileKind,
    frontend::{RenameAction, UploadAction},
};
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

    let mut data = send_req!(req, Directory(query.path.clone()))?;

    data.dir_list.sort_by(|a, b| a.path.cmp(&b.path));

    let content = html! {
        #browser-swap nm-data="selectedRow: null, viewHidden: false" {
            (path_display(&query.path))

            table #browser-inner {
                tr {
                    th { "File Name" }
                    th { "File Size" }
                }
                @for item in data.dir_list {
                    @let name = item.path.rsplit_once('/').map(|(_, name)| name).unwrap_or(&item.path);
                    @let is_hidden = name.starts_with('.');
                    @let icon = match item.kind {
                        FileKind::TextFile => "fa6-solid-file-lines",
                        FileKind::BinaryFile => "fa6-solid-file",
                        FileKind::Directory => "fa6-solid-folder",
                        FileKind::Special => "fa6-solid-cube",
                    };
                    @let pretty_size = item.size.map(|size| pretty_bytes(size, Some(0)).to_string()).unwrap_or_else(|| "--".into());
                    @let dblclick = match item.kind {
                        FileKind::TextFile => "get('/browser/file', {path});",
                        FileKind::Directory => "get('/browser', {path});",
                        FileKind::BinaryFile => "window.open(`/browser/actions/download?path=${path}`)",
                        FileKind::Special => "",
                    };
                    tr
                        data-current-path=(query.path)
                        data-path=(item.path)
                        data-kind=(serde_plain::to_string(&item.kind).unwrap())
                        data-hidden[is_hidden]
                        nm-bind={"
                            ariaCurrent: () => selectedRow === this,
                            onclick: () => {
                                let {currentPath, path, kind} = this.dataset;
                                selectedRow = this;
                                get('/browser/actions', {currentPath, path, kind})
                            },
                            ondblclick: () => {
                                let {path} = this.dataset;
                                "(dblclick)"
                            },
                            hidden: () => this.hasAttribute('data-hidden') && !viewHidden
                        "}
                    {
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

pub async fn file(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: BrowserQuery = req.extract_query()?;

    let data = send_req!(req, Download(query.path.clone()))?;
    let data = String::from_utf8(data).map_err(|_| {
        ServerResponse::new()
            .status(StatusCode::BAD_REQUEST)
            .body("not a text file")
    })?;

    let content = html! {
        #browser-swap nm-data="data: ''" {
            (path_display(&query.path))

            code-editor #browser-inner {
                textarea nm-bind="oninput: () => data = this.value" {
                    (data)
                }
                pre {}
            }
            #actions-list {
                button title="Save" nm-bind={"
                    onclick: () => {
                        post('/browser/file/save', {path: '"(query.path)"', data});
                    }
                "} {
                    (Icon::new("fa6-solid-floppy-disk"))
                }
            }
        }
    };

    template(&req, content)
}

fn path_display(path: &str) -> Markup {
    let mut paths = path
        .split_inclusive('/')
        .scan(String::new(), |acc, segment| {
            acc.push_str(segment);
            Some((acc.clone(), segment))
        })
        .peekable();

    html! {
        #path-display {
            @while let Some((full_path, path_segment)) = paths.next() {
                @if paths.peek().is_none() {
                    span { (path_segment) }
                } @else {
                    button data-path=(full_path) nm-bind={"onclick: () => get('/browser', {path: this.dataset.path})"} { (path_segment) }
                }
            }
        }
    }
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
                button title="Download" nm-bind={"
                    onclick: () => { 
                        window.open('/browser/actions/download?path="(query.path)"')
                    }
                "} { (Icon::new("fa6-solid-file-arrow-down")) }
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
        button title="Hide Hidden Files" nm-bind="
            onclick: () => viewHidden = true,
            hidden: () => viewHidden,
        " { (Icon::new("fa6-solid-eye")) }
        button title="Show Hidden Files" nm-bind="
            onclick: () => viewHidden = false,
            hidden: () => !viewHidden,
        " { (Icon::new("fa6-solid-eye-slash")) }
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
        button title="Upload" onclick="this.firstChild.click()" {
            input type="file" hidden nm-bind={"
                onchange: () => {
                    let file = this.files[0];
                    if (!file) return;
    
                    let reader = new FileReader();
                    reader.readAsDataURL(file);
                    reader.addEventListener('load', () => {
                        let data = reader.result.replace(/^data:.*;base64,/, '');
    
                        post('/browser/actions/upload', {
                            parent: '"(current_path)"',
                            name: file.name,
                            data
                        })
                    });
                }
            "};
            (Icon::new("fa6-solid-file-arrow-up"))
        }
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

    let mut path = PathBuf::from(&query.parent);
    path.push(query.name);
    let path = path.to_str().unwrap();

    send_act!(req, NewFile(path.into()))?;

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
    let parent = parent.to_str().unwrap();

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
    let parent = parent.to_str().unwrap();

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
    let parent = parent.to_str().unwrap();

    send_act!(req, DeleteFolder(query.path.clone()))?;

    Ok(ServerResponse::new().redirect(RedirectType::SeeOther, &format!("/browser?path={parent}")))
}

pub async fn download(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: FileQuery = req.extract_query()?;

    let data = send_req!(req, Download(query.path))?;

    Ok(ServerResponse::new().body(data))
}

#[derive(Deserialize)]
pub struct SaveForm {
    path: String,
    data: String,
}

pub async fn save(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: SaveForm = req.extract_form().await?;

    let action = UploadAction {
        path: query.path,
        data: query.data.into_bytes(),
    };
    send_act!(req, Upload(action))?;

    Ok(ServerResponse::new())
}

#[derive(Deserialize)]
pub struct UploadForm {
    name: String,
    parent: String,
    data: String,
}

pub async fn upload(mut req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: UploadForm = req.extract_form().await?;

    let mut path = PathBuf::from(&query.parent);
    path.push(query.name);
    let path = path.to_str().unwrap();

    let data = BASE64.decode(query.data.as_bytes()).map_err(|_| {
        ServerResponse::new()
            .status(StatusCode::BAD_REQUEST)
            .body("invalid base64")
    })?;

    let action = UploadAction {
        path: path.into(),
        data,
    };
    send_act!(req, Upload(action))?;

    Ok(ServerResponse::new().redirect(
        RedirectType::SeeOther,
        &format!("/browser?path={}", query.parent),
    ))
}

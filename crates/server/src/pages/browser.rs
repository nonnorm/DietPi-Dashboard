use maud::html;
use pretty_bytes_typed::pretty_bytes;
use proto::backend::FileKind;
use serde::Deserialize;

use crate::{
    http::{request::ServerRequest, response::ServerResponse},
    pages::template::Icon,
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
        #browser-swap {
            #path-display {
                @let paths = query.path.split_inclusive('/').scan(String::new(), |acc, segment| {
                    acc.push_str(segment);
                    Some((acc.clone(), segment))
                });

                @for (full_path, path_segment) in paths {
                    button nm-bind={"onclick: () => get('/browser?path="(full_path)"')"} { (path_segment) }
                }
            }

            table #browser-table nm-data="selectedRow: null" {
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
                        onclick: () => {
                            selectedRow = this;
                        },
                        ondblclick: () => {
                            get('/browser?path="(item.path)"');
                        }
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
            }
        }
    };

    template(&req, content)
}

use maud::{Markup, html};
use pretty_bytes_typed::pretty_bytes_binary;
use proto::{
    backend::ProcessStatus,
    frontend::{ActionFrontendMessage, SignalAction},
};
use serde::{Deserialize, Serialize};

use crate::http::{request::ServerRequest, response::ServerResponse};

use super::template::{Icon, send_req, template};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ProcessQuery {
    sort: ColumnSort,
    reverse: bool,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColumnSort {
    #[default]
    Pid,
    Name,
    Status,
    Cpu,
    Ram,
}

fn table_header(name: &str, sort: ColumnSort, query: &ProcessQuery) -> Markup {
    let reverse = if query.sort == sort {
        !query.reverse
    } else {
        false
    };

    let new_query = ProcessQuery { sort, reverse };
    let new_query = serde_urlencoded::to_string(&new_query).unwrap();

    let url = format!("'/process?{new_query}'");

    html! {
        th {
            button nm-bind={ "onclick: () => get("(url)")" } {
                (name)
                @if query.sort == sort {
                    @if query.reverse {
                        (Icon::new("fa6-solid-sort-down"))
                    } @else {
                        (Icon::new("fa6-solid-sort-up"))
                    }
                }
            }
        }
    }
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let query: ProcessQuery = req.extract_query()?;

    let mut processes = send_req!(req, Processes)?.processes;
    match query.sort {
        ColumnSort::Pid => processes.sort_by_key(|a| a.pid),
        ColumnSort::Name => processes.sort_by(|a, b| a.name.cmp(&b.name)),
        ColumnSort::Status => processes.sort_by_key(|a| a.status),
        ColumnSort::Cpu => processes.sort_by(|a, b| a.cpu.total_cmp(&b.cpu)),
        ColumnSort::Ram => processes.sort_by_key(|a| a.mem),
    }
    if query.reverse {
        processes.reverse();
    }

    let query_str = serde_urlencoded::to_string(&query).unwrap();
    let url = format!("'/process?{query_str}'",);

    let content = html! {
        section #process-swap nm-bind={ "_: () => debounce(() => get("(url)"), 2000)" } {
            h2 { "Processes" }

            table .process-table {
                tr {
                    (table_header("PID", ColumnSort::Pid, &query))
                    (table_header("Name", ColumnSort::Name, &query))
                    (table_header("Status", ColumnSort::Status, &query))
                    (table_header("CPU Usage", ColumnSort::Cpu, &query))
                    (table_header("RAM Usage", ColumnSort::Ram, &query))
                    th { "Actions" }
                }
                @for proc in processes {
                    @let pretty_mem = pretty_bytes_binary(proc.mem, Some(0));

                    tr {
                        td { (proc.pid) }
                        td { (proc.name) }
                        td { (format!("{:?}", proc.status)) }
                        td { (proc.cpu) "%" }
                        td { (pretty_mem) }
                        td {
                            .actions-cell {
                                button nm-bind={ "onclick: () => post('/process/signal?signal=kill&pid="(proc.pid)"')" } {
                                    (Icon::new("fa6-solid-skull"))
                                }
                                button nm-bind={ "onclick: () => post('/process/signal?signal=term&pid="(proc.pid)"')" } {
                                    (Icon::new("fa6-solid-ban"))
                                }
                                @if proc.status == ProcessStatus::Paused {
                                    button nm-bind={ "onclick: () => post('/process/signal?signal=resume&pid="(proc.pid)"')" } {
                                        (Icon::new("fa6-solid-play"))
                                    }
                                } @else {
                                    button nm-bind={ "onclick: () => post('/process/signal?signal=pause&pid="(proc.pid)"')" } {
                                        (Icon::new("fa6-solid-pause"))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    template(&req, content)
}

pub async fn signal(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let signal: SignalAction = req.extract_query()?;

    req.send_backend_action(ActionFrontendMessage::Signal(signal))
        .await?;

    Ok(ServerResponse::new())
}

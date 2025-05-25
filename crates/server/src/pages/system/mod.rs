use maud::html;
use serde::{Deserialize, Serialize};

use crate::http::{query_array::QueryArray, request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

mod fragments;
mod graph;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemQuery {
    cpu_points: QueryArray,
    temp_points: QueryArray,
    ram_points: QueryArray,
    swap_points: QueryArray,
    sent_points: QueryArray,
    recv_points: QueryArray,
}

pub async fn page(req: ServerRequest) -> Result<ServerResponse, ServerResponse> {
    req.check_login()?;

    let mut query: SystemQuery = req.extract_query()?;

    let cpu_data = send_req!(req, Cpu)?;
    let temp_data = send_req!(req, Temp)?;
    let mem_data = send_req!(req, Mem)?;
    let disk_data = send_req!(req, Disk)?;
    let net_data = send_req!(req, NetIO)?;

    let cpu_meters = fragments::cpu_meters(&cpu_data, &temp_data);
    let mem_meters = fragments::mem_meters(&mem_data);
    let disk_meters = fragments::disk_meters(&disk_data);

    let cpu_graph = fragments::cpu_graph(&cpu_data, &mut query.cpu_points);
    let temp_graph = fragments::temp_graph(&temp_data, &mut query.temp_points);
    let mem_graph = fragments::mem_graph(&mem_data, &mut query.ram_points, &mut query.swap_points);
    let net_graph = fragments::net_graph(&net_data, &mut query.sent_points, &mut query.recv_points);

    let new_query = serde_urlencoded::to_string(&query).unwrap();

    let content = html! {
        server-swap .card-grid action={"/system?" (new_query)} trigger="delay" {
            (cpu_meters)
            (cpu_graph)
            @if let Some(temp_graph) = temp_graph {
                (temp_graph)
            }
            (mem_meters)
            (mem_graph)
            (disk_meters)
            (net_graph)
        }
    };

    template(&req, content)
}

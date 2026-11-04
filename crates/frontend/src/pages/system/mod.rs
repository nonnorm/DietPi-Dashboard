use maud::html;
use pretty_bytes_typed::{pretty_bytes, pretty_bytes_binary};
use serde::{Deserialize, Serialize};

use crate::http::{query_array::QueryArray, request::ServerRequest, response::ServerResponse};

use super::template::{send_req, template};

mod fragments;
mod graph;

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemQuery {
    cpu: QueryArray,
    temp: QueryArray,
    ram: QueryArray,
    swap: QueryArray,
    sent: QueryArray,
    recv: QueryArray,
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

    let cpu_graph = fragments::cpu_graph(&cpu_data, &mut query.cpu);
    let temp_graph = fragments::temp_graph(&temp_data, &mut query.temp);
    let mem_graph = fragments::mem_graph(&mem_data, &mut query.ram, &mut query.swap);
    let net_graph = fragments::net_graph(&net_data, &mut query.sent, &mut query.recv);

    let ram_percent = if mem_data.ram.total == 0 {
        0.
    } else {
        mem_data.ram.used as f32 / mem_data.ram.total as f32 * 100.
    };
    let swap_percent = if mem_data.swap.total == 0 {
        0.
    } else {
        mem_data.swap.used as f32 / mem_data.swap.total as f32 * 100.
    };
    let temp_display = temp_data
        .temp
        .map(|temp| format!("{temp:.1}ºC"))
        .unwrap_or_else(|| "--".to_string());

    let peak_disk = disk_data.disks.iter().max_by(|a, b| {
        let a_ratio = if a.usage.total == 0 {
            0.
        } else {
            a.usage.used as f32 / a.usage.total as f32
        };
        let b_ratio = if b.usage.total == 0 {
            0.
        } else {
            b.usage.used as f32 / b.usage.total as f32
        };

        a_ratio.total_cmp(&b_ratio)
    });

    let peak_disk_display = peak_disk
        .map(|disk| {
            let disk_percent = if disk.usage.total == 0 {
                0.
            } else {
                disk.usage.used as f32 / disk.usage.total as f32 * 100.
            };

            format!("{} ({disk_percent:.1}%)", disk.name)
        })
        .unwrap_or_else(|| "--".to_string());

    let ram_usage = format!(
        "{} / {}",
        pretty_bytes_binary(mem_data.ram.used, Some(1)),
        pretty_bytes_binary(mem_data.ram.total, Some(1))
    );
    let net_usage = format!(
        "↑ {}  ↓ {}",
        pretty_bytes(net_data.sent, Some(1)),
        pretty_bytes(net_data.recv, Some(1))
    );

    let content = html! {
            div #system-swap
                nm-bind="oninit: () => $debounce(() => $get('/system'), 2000)"
                data-cpu=(query.cpu)
                data-ram=(query.ram)
                data-swap=(query.swap)
                data-temp=(query.temp)
                data-sent=(query.sent)
                data-recv=(query.recv)
            {
                section #system-overview {
                    h2 data-i18n="system_overview" { "System Overview" }
                    .kpi-grid {
                        article .kpi-card {
                            p .kpi-label data-i18n="cpu_load" { "CPU Load" }
                            p .kpi-value { (format!("{:.1}%", cpu_data.global_cpu)) }
                            p .kpi-sub data-i18n-template="temperature_value" data-value=(temp_display) { "Temperature: " (temp_display) }
                        }
                        article .kpi-card {
                            p .kpi-label data-i18n="memory_label" { "Memory" }
                            p .kpi-value { (format!("{ram_percent:.1}%")) }
                            p .kpi-sub { (ram_usage) }
                        }
                        article .kpi-card {
                            p .kpi-label data-i18n="swap_label" { "Swap" }
                            p .kpi-value { (format!("{swap_percent:.1}%")) }
                            p .kpi-sub data-i18n="pressure_monitor" { "Pressure monitor" }
                        }
                        article .kpi-card {
                            p .kpi-label data-i18n="peak_disk" { "Peak Disk" }
                            p .kpi-value { (peak_disk_display) }
                            p .kpi-sub { (net_usage) }
                        }
                    }
                }

                .system-layout {
                    .system-metrics {
                        (cpu_meters)
                        (mem_meters)
                        (disk_meters)
                    }
                    .system-graphs {
                        (cpu_graph)
                        @if let Some(temp_graph) = temp_graph {
                            (temp_graph)
                        }
                        (mem_graph)
                        (net_graph)
                    }
                }
            }
    };

    template(&req, content, "x: null, idx: 0")
}

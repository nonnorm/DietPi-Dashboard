use maud::{Markup, html};
use pretty_bytes_typed::{pretty_bytes, pretty_bytes_binary};
use proto::backend::{CpuResponse, DiskResponse, MemResponse, NetworkResponse, TempResponse};

use crate::http::query_array::QueryArray;

use super::graph::{Axis, SvgGraph};

fn calc_percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.;
    };

    let percent = used as f32 / total as f32 * 100.;
    // Round percent to 2 decimal places
    (percent * 100.).round() / 100.
}

pub fn cpu_meters(cpu_data: &CpuResponse, temp_data: &TempResponse) -> Markup {
    let cpu_iter = cpu_data.cpus.iter().zip(1_u8..);

    html! {
        section {
            h2 data-i18n="cpu_statistics" { "CPU Statistics" }
            @if let Some(temp) = temp_data.temp {
                @let temp_text = format!("{temp:.1}ºC");
                p data-i18n-template="cpu_temperature_value" data-value=(temp_text) { "CPU Temperature: " (format!("{temp:.1}")) "ºC" }
            }
            @let global_cpu_text = format!("{:.1}%", cpu_data.global_cpu);
            p data-i18n-template="global_cpu_value" data-value=(global_cpu_text) { "Global CPU: " (format!("{:.1}", cpu_data.global_cpu)) "%" }
            .meter-container {
                .bar.cpu style={"--scale:"(cpu_data.global_cpu / 100.)} {}
            }
            @for (usage, num) in cpu_iter {
                @let core_cpu_text = format!("{usage:.1}%");
                p data-i18n-template="cpu_core_usage" data-core=(num) data-value=(core_cpu_text) { "CPU "(num)": " (format!("{usage:.1}")) "%" }
                .meter-container {
                    .bar.cpu style={"--scale:"(usage / 100.)} {}
                }
            }
        }
    }
}

pub fn cpu_graph(data: &CpuResponse, points: &mut QueryArray) -> Markup {
    let mut graph = SvgGraph::new(Axis::Percent);

    let points_iter = std::iter::once(data.global_cpu)
        .chain(points.iter())
        .take(20);

    graph.add_series(points_iter.clone(), "var(--gray-12)", "CPU", "cpu");

    *points = points_iter.collect();

    html! {
        section {
            h2 data-i18n="cpu_graph" { "CPU Graph" }
            (graph)
        }
    }
}

pub fn temp_graph(data: &TempResponse, points: &mut QueryArray) -> Option<Markup> {
    data.temp.map(|temp| {
        let mut graph = SvgGraph::new(Axis::Temp);

        let points_iter = std::iter::once(temp).chain(points.iter()).take(20);

        graph.add_series(points_iter.clone(), "var(--red-6)", "Temperature", "temperature");

        *points = points_iter.collect();

        html! {
            section {
                h2 data-i18n="temperature_graph" { "Temperature Graph" }
                (graph)
            }
        }
    })
}

pub fn mem_meters(data: &MemResponse) -> Markup {
    let pretty_ram_used = pretty_bytes_binary(data.ram.used, Some(2));
    let pretty_ram_total = pretty_bytes_binary(data.ram.total, Some(2));
    let ram_percent = calc_percent(data.ram.used, data.ram.total);

    let pretty_swap_used = pretty_bytes_binary(data.swap.used, Some(2));
    let pretty_swap_total = pretty_bytes_binary(data.swap.total, Some(2));
    let swap_percent = calc_percent(data.swap.used, data.swap.total);

    html! {
        section {
            h2 data-i18n="memory_usage" { "Memory Usage" }

            @let ram_usage_text = format!("{pretty_ram_used} / {pretty_ram_total}");
            p data-i18n-template="ram_usage_value" data-value=(ram_usage_text) { "RAM Usage: " (pretty_ram_used) " / " (pretty_ram_total) }
            div .meter-container {
                div .bar.ram style={"--scale:"(ram_percent / 100.)} {}
            }

            @let swap_usage_text = format!("{pretty_swap_used} / {pretty_swap_total}");
            p data-i18n-template="swap_usage_value" data-value=(swap_usage_text) { "Swap Usage: " (pretty_swap_used) " / " (pretty_swap_total) }
            div .meter-container {
                div .bar.swap style={"--scale:"(swap_percent / 100.)} {}
            }
        }
    }
}

pub fn mem_graph(
    data: &MemResponse,
    ram_points: &mut QueryArray,
    swap_points: &mut QueryArray,
) -> Markup {
    let mut graph = SvgGraph::new(Axis::Percent);

    let ram_percent = calc_percent(data.ram.used, data.ram.total);
    let swap_percent = calc_percent(data.swap.used, data.swap.total);

    let ram_points_iter = std::iter::once(ram_percent)
        .chain(ram_points.iter())
        .take(20);
    let swap_points_iter = std::iter::once(swap_percent)
        .chain(swap_points.iter())
        .take(20);

    graph.add_series(ram_points_iter.clone(), "var(--gray-12)", "RAM", "ram");
    graph.add_series(swap_points_iter.clone(), "var(--red-6)", "Swap", "swap");

    *ram_points = ram_points_iter.collect();
    *swap_points = swap_points_iter.collect();

    html! {
        section {
            h2 data-i18n="memory_graph" { "Memory Graph" }
            (graph)
        }
    }
}

pub fn disk_meters(data: &DiskResponse) -> Markup {
    html! {
        section {
            h2 data-i18n="disk_usage" { "Disk Usage" }

            @for disk in &data.disks {
                @let pretty_disk_used = pretty_bytes(disk.usage.used, Some(2));
                @let pretty_disk_total = pretty_bytes(disk.usage.total, Some(2));
                @let disk_percent = calc_percent(disk.usage.used, disk.usage.total);
                @let disk_usage_text = format!("{pretty_disk_used} / {pretty_disk_total}");

                p data-i18n-template="disk_usage_value" data-name=(disk.name) data-mount=(disk.mnt_point) data-value=(disk_usage_text) { (disk.name) " (" (disk.mnt_point) "): " (pretty_disk_used) " / " (pretty_disk_total) }
                .meter-container {
                    .bar.disk style={"--scale:"(disk_percent / 100.)} {}
                }
            }
        }
    }
}

pub fn net_graph(
    data: &NetworkResponse,
    sent_points: &mut QueryArray,
    recv_points: &mut QueryArray,
) -> Markup {
    let mut graph = SvgGraph::new(Axis::Bytes);

    let sent_points_iter = std::iter::once(data.sent as f32)
        .chain(sent_points.iter())
        .take(20);
    let recv_points_iter = std::iter::once(data.recv as f32)
        .chain(recv_points.iter())
        .take(20);

    graph.add_series(sent_points_iter.clone(), "var(--gray-12)", "Sent", "sent");
    graph.add_series(recv_points_iter.clone(), "var(--red-6)", "Received", "received");

    *sent_points = sent_points_iter.collect();
    *recv_points = recv_points_iter.collect();

    html! {
        section {
            h2 data-i18n="network_graph" { "Network Graph" }
            (graph)
        }
    }
}

use std::{fs, path::PathBuf, process::Command};

use proto::{
    backend::{
        CommandResponse, CpuResponse, DiskInfo, DiskResponse, HostResponse, MemResponse,
        NetworkResponse, ProcessInfo, ProcessResponse, ProcessStatus, ServiceInfo, ServiceResponse,
        ServiceStatus, SoftwareInfo, SoftwareResponse, TempResponse, UsageData,
    },
    frontend::CommandAction,
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::client::BackendContext;

fn round_to_2(num: f32) -> f32 {
    (num * 100.).round() / 100.
}

pub fn cpu(mut ctx: BackendContext) -> CpuResponse {
    let sys = &mut ctx.system().system;

    sys.refresh_cpu_usage();

    let global_cpu = round_to_2(sys.global_cpu_usage());
    let cpus: Vec<f32> = sys
        .cpus()
        .iter()
        .map(|x| round_to_2(x.cpu_usage()))
        .collect();

    CpuResponse { global_cpu, cpus }
}

pub fn temp(mut ctx: BackendContext) -> TempResponse {
    let components = &mut ctx.system().components;
    components.refresh();
    let components = components.list();

    let known_sensor_names = ["coretemp Package", "tdie"];

    let temp = components
        .iter()
        .find(|x| known_sensor_names.iter().any(|y| x.label().contains(y)))
        .or_else(|| components.first())
        .map(|x| round_to_2(x.temperature()));

    TempResponse { temp }
}

pub fn memory(mut ctx: BackendContext) -> MemResponse {
    let sys = &mut ctx.system().system;

    // Refreshes both RAM and Swap
    sys.refresh_memory();

    let ram = UsageData {
        used: sys.used_memory(),
        total: sys.total_memory(),
    };

    let swap = UsageData {
        used: sys.used_swap(),
        total: sys.total_swap(),
    };

    MemResponse { ram, swap }
}

pub fn disks(mut ctx: BackendContext) -> DiskResponse {
    let mnt_points = &ctx.config.disks;
    let mnt_points: Vec<_> = mnt_points.iter().map(PathBuf::from).collect();

    let disks = &mut ctx.system().disks;
    disks.refresh();
    let disks = disks.list();

    let disks: Vec<_> = disks
        .iter()
        .filter(|disk| mnt_points.iter().any(|path| path == disk.mount_point()))
        .map(|disk| DiskInfo {
            name: disk.name().to_str().unwrap_or("unknown").into(),
            mnt_point: disk.mount_point().to_str().unwrap_or("unknown").into(),
            usage: UsageData {
                used: disk.total_space() - disk.available_space(),
                total: disk.total_space(),
            },
        })
        .collect();

    DiskResponse { disks }
}

pub fn network_io(mut ctx: BackendContext) -> NetworkResponse {
    let networks = &mut ctx.system().networks;
    networks.refresh();
    let networks = networks.list();

    let mut resp = NetworkResponse { sent: 0, recv: 0 };

    for net in networks.values() {
        resp.recv += net.received();
        resp.sent += net.transmitted();
    }

    resp
}

pub fn processes(mut ctx: BackendContext) -> ProcessResponse {
    let sys = &mut ctx.system().system;

    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new()
            .with_cpu()
            .with_memory()
            .with_cmd(UpdateKind::OnlyIfNotSet),
    );

    let processes = sys
        .processes()
        .iter()
        .filter(|(_, proc)| !proc.cmd().is_empty())
        .map(|(pid, proc)| ProcessInfo {
            pid: pid.as_u32(),
            name: proc.name().to_string_lossy().into(),
            cpu: round_to_2(proc.cpu_usage()),
            mem: proc.memory(),
            status: match proc.status() {
                sysinfo::ProcessStatus::Run => ProcessStatus::Running,
                sysinfo::ProcessStatus::Sleep => ProcessStatus::Sleeping,
                sysinfo::ProcessStatus::Stop => ProcessStatus::Paused,
                _ => ProcessStatus::Other,
            },
        })
        .collect();

    ProcessResponse { processes }
}

pub fn host(mut ctx: BackendContext) -> HostResponse {
    let net = &ctx.system().networks;

    let unknown = || "unknown".to_string();

    let nic = net
        .iter()
        .max_by_key(|(_, net)| net.total_transmitted())
        .map(|(name, _)| name)
        .cloned()
        .unwrap_or_else(unknown);

    let uptime = System::uptime();
    let arch = System::cpu_arch().unwrap_or_else(unknown);
    let os_version = System::long_os_version().unwrap_or_else(unknown);
    let kernel = System::kernel_version().unwrap_or_else(unknown);
    let hostname = System::host_name().unwrap_or_else(unknown);

    let dp_file = fs::read_to_string("/boot/dietpi/.version").ok();
    let dp_version = dp_file
        .and_then(|file| {
            let mut fields = file.split(['=', '\n']);

            // Extract items 1, 3, and 5
            Some(format!(
                "{}.{}.{}",
                fields.nth(1)?,
                fields.nth(1)?,
                fields.nth(1)?
            ))
        })
        .unwrap_or_else(unknown);

    let pkg_list = Command::new("dpkg").arg("--get-selections").output().ok();
    let num_pkgs = pkg_list
        .map(|output| output.stdout.into_iter().filter(|&x| x == b'\n').count())
        .unwrap_or(0);

    HostResponse {
        nic,
        uptime,
        arch,
        os_version,
        kernel,
        hostname,
        dp_version,
        num_pkgs,
    }
}

fn parse_software_line(line: &str) -> Option<(SoftwareInfo, bool)> {
    let mut fields = line.split('|');

    let id = fields.next()?;
    if id.contains("DISABLED") {
        return None;
    }
    let id: u16 = id.parse().ok()?;

    let installed = fields.next()?;
    let installed: u8 = installed.parse().ok()?;
    let installed = installed > 0;

    let name = fields.next()?.into();

    let desc = fields.next()?.into();

    let deps = fields.next()?.into();

    let docs = fields.next()?.into();

    Some((
        SoftwareInfo {
            id,
            name,
            desc,
            deps,
            docs,
        },
        installed,
    ))
}

pub fn software(_ctx: BackendContext) -> SoftwareResponse {
    let cmd_out = Command::new("/boot/dietpi/dietpi-software")
        .args(["list", "--machine-readable"])
        .output()
        .ok();
    let cmd_out = cmd_out.and_then(|output| String::from_utf8(output.stdout).ok());

    let software_iter = cmd_out
        .iter()
        .flat_map(|out| out.lines())
        .filter_map(parse_software_line);

    let mut resp = SoftwareResponse {
        installed: Vec::new(),
        uninstalled: Vec::new(),
    };

    for (info, installed) in software_iter {
        if installed {
            resp.installed.push(info);
        } else {
            resp.uninstalled.push(info);
        }
    }

    resp
}

fn remove_escape_codes(s: impl Iterator<Item = u8>) -> Vec<u8> {
    s.scan(false, |in_escape, c| {
        if *in_escape {
            if c.is_ascii_alphabetic() {
                *in_escape = false;
            }
            Some(None)
        } else if c == b'\x1b' {
            *in_escape = true;
            Some(None)
        } else {
            Some(Some(c))
        }
    })
    .flatten()
    .collect()
}

pub fn command(_ctx: BackendContext, action: CommandAction) -> CommandResponse {
    let output = Command::new(action.cmd)
        .args(&action.args)
        .output()
        .map(|out| out.stdout.into_iter())
        .map(remove_escape_codes)
        .unwrap_or_else(|err| format!("command execution failed: {err}").into());

    CommandResponse { output }
}

fn services_helper() -> Option<ServiceResponse> {
    let output = Command::new("/boot/dietpi/dietpi-services")
        .arg("status")
        .output()
        .ok()?;

    let stdout = remove_escape_codes(output.stdout.into_iter());
    let stdout = std::str::from_utf8(&stdout).ok()?;

    let stderr = remove_escape_codes(output.stderr.into_iter());
    let stderr = std::str::from_utf8(&stderr).ok()?;

    let ok_services = stdout
        .lines()
        .map(|line| {
            line.trim_start_matches("[  OK  ]")
                .trim_start_matches("[ INFO ]")
                .trim_start_matches(" DietPi-Services |")
        })
        .filter_map(|line| line.split_once('\t'))
        .map(|(name, statusdate)| {
            let statusdate = statusdate.trim();
            let (status, date) = statusdate.split_once(" since ").unzip();
            (name, status.unwrap_or(statusdate), date.unwrap_or_default())
        })
        .map(|(name, status, date)| {
            let status = match status {
                "active (running)" | "active (exited)" => ServiceStatus::Active,
                "inactive (dead)" => ServiceStatus::Inactive,
                _ => ServiceStatus::Unknown,
            };

            ServiceInfo {
                name: name.into(),
                status,
                start: date.into(),
                err_log: String::new(),
            }
        });

    let failed_services = stderr
        .split("[FAILED] DietPi-Services | ×")
        .filter_map(|desc| desc.split_once(".service"))
        .filter_map(|(name, rest)| rest.split_once("\n\n").map(|(_, err_log)| (name, err_log)))
        .map(|(name, err_log)| ServiceInfo {
            name: name.into(),
            status: ServiceStatus::Failed,
            start: String::new(),
            err_log: err_log.into(),
        });

    let mut services = Vec::new();
    services.extend(ok_services);
    services.extend(failed_services);

    Some(ServiceResponse { services })
}

pub fn services(_ctx: BackendContext) -> ServiceResponse {
    services_helper().unwrap_or_else(|| ServiceResponse {
        services: Vec::new(),
    })
}

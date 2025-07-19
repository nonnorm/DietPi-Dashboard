use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Encode, Decode)]
pub enum BackendMessage {
    Action(ActionBackendMessage),
    Response(u16, ResponseBackendMessage),
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum ActionBackendMessage {
    Handshake(Handshake),
    Terminal(Vec<u8>),
}

#[derive(Debug, Clone, Encode, Decode)]
pub enum ResponseBackendMessage {
    Cpu(CpuResponse),
    Temp(TempResponse),
    Mem(MemResponse),
    Disk(DiskResponse),
    NetIO(NetworkResponse),
    Processes(ProcessResponse),
    Host(HostResponse),
    Software(SoftwareResponse),
    Command(CommandResponse),
    Services(ServiceResponse),
    Directory(DirectoryResponse),
    Download(Vec<u8>),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Handshake {
    pub nickname: String,
    pub version: u32,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CpuResponse {
    pub global_cpu: f32,
    pub cpus: Vec<f32>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TempResponse {
    pub temp: Option<f32>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct MemResponse {
    pub ram: UsageData,
    pub swap: UsageData,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UsageData {
    pub used: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DiskResponse {
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DiskInfo {
    pub name: String,
    pub mnt_point: String,
    pub usage: UsageData,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct NetworkResponse {
    pub sent: u64,
    pub recv: u64,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ProcessResponse {
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub mem: u64,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessStatus {
    Running,
    Paused,
    Sleeping,
    Other,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct HostResponse {
    pub hostname: String,
    pub nic: String,
    pub arch: String,
    pub uptime: u64,
    pub kernel: String,
    pub os_version: String,
    pub dp_version: String,
    pub num_pkgs: usize,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SoftwareResponse {
    pub installed: Vec<SoftwareInfo>,
    pub uninstalled: Vec<SoftwareInfo>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SoftwareInfo {
    pub id: u16,
    pub name: String,
    pub desc: String,
    pub deps: String,
    pub docs: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CommandResponse {
    pub output: Vec<u8>,
}

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct ServiceResponse {
    pub services: Vec<ServiceInfo>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub start: String,
    pub err_log: String,
}

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub enum ServiceStatus {
    Active,
    Inactive,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct DirectoryResponse {
    pub dir_list: Vec<DirectoryItemInfo>,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct DirectoryItemInfo {
    pub path: String,
    pub kind: FileKind,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Copy, Encode, Decode, Deserialize, Serialize, PartialEq, Eq)]
pub enum FileKind {
    TextFile,
    BinaryFile,
    Directory,
    Special,
}

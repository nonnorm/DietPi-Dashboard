use bitcode::{Decode, Encode};
use serde::Deserialize;

#[derive(Debug, Encode, Decode)]
pub enum FrontendMessage {
    Request(u16, RequestFrontendMessage),
    Action(ActionFrontendMessage),
}

#[derive(Debug, Encode, Decode)]
pub enum RequestFrontendMessage {
    Cpu,
    Temp,
    Mem,
    Disk,
    NetIO,
    Processes,
    Host,
    Software,
    Command(CommandAction),
    Services,
    Directory(String),
    Download(String),
}

#[derive(Debug, Encode, Decode)]
pub enum ActionFrontendMessage {
    Terminal(Vec<u8>),
    Signal(SignalAction),
    NewFile(String),
    NewFolder(String),
    Rename(RenameAction),
    DeleteFile(String),
    DeleteFolder(String),
    Upload(UploadAction),
}

#[derive(Debug, Encode, Decode, Deserialize)]
pub struct SignalAction {
    pub pid: u32,
    pub signal: Signal,
}

#[derive(Debug, Encode, Decode, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Signal {
    Term,
    Pause,
    Resume,
    Kill,
}

#[derive(Debug, Encode, Decode)]
pub struct CommandAction {
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Debug, Encode, Decode)]
pub struct RenameAction {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Encode, Decode)]
pub struct UploadAction {
    pub path: String,
    pub data: Vec<u8>,
}

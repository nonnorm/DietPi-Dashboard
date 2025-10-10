use anyhow::Context;
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};

pub static CONFIG: std::sync::LazyLock<crate::config::Config> =
    std::sync::LazyLock::new(crate::config::config);

// Simple error handling macro, print out error and source (if available), and handle error if it exists
#[macro_export]
macro_rules! handle_error {
    ($e: expr $(, $handler:expr)?) => {
        match $e {
            Ok(val) => val,
            Err(err) => {
                tracing::warn!("{:#}", err);
                $($handler)?
            }
        }
    };
}

pub fn remove_color_codes(s: &str) -> String {
    s.replace('\u{1b}', "")
        .replace("[33m", "")
        .replace("[90m", "")
        .replace("[0m", "")
        .replace("[32m", "")
        .replace("[31m", "")
        .replace("[38;5;154m", "")
        .replace("[J", "")
}

pub fn get_token_from_list<'a>(
    token_list: &'a str,
    pat: [char; 2],
    token_name: &str,
) -> Option<&'a str> {
    let mut iter = token_list.split(pat).map(str::trim);
    if !iter.any(|x| x == token_name) {
        return None;
    }
    if let Some(next) = iter.next() {
        return Some(next);
    }
    None
}

pub fn get_fingerprint(req: &hyper::Request<hyper::Body>) -> anyhow::Result<Option<String>> {
    let cookie = if let Some(cookie) = req.headers().get(hyper::header::COOKIE) {
        cookie.to_str().context("Invalid cookie list")?
    } else {
        return Ok(None);
    };
    let fingerprint_option = get_token_from_list(cookie, ['=', ';'], "FINGERPRINT");
    Ok(Some(hex::encode(ring::digest::digest(
        &ring::digest::SHA256,
        &hex::decode(if let Some(fingerprint) = fingerprint_option {
            fingerprint
        } else {
            return Ok(None);
        })
        .context("Invalid fingerprint token")?,
    ))))
}

pub struct SocketSend(
    pub  futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<hyper::upgrade::Upgraded>,
        tokio_tungstenite::tungstenite::Message,
    >,
);

impl SocketSend {
    pub async fn send(&mut self, value: BackendData) -> anyhow::Result<()> {
        Ok(self
            .0
            .send(tokio_tungstenite::tungstenite::Message::Text(
                serde_json::to_string(&value).context("Couldn't serialize JSON")?,
            ))
            .await?)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "UPPERCASE")]
#[serde(tag = "dataKind")]
pub enum BackendData {
    Statistic(SysData),
    Process(ProcessList),
    Software(DPSoftwareList),
    Management(HostData),
    Service(ServiceList),
    Global(GlobalData),
    Browser(BrowserList),
    Reauth,
}

#[derive(Serialize, Default)]
pub struct SysData {
    pub cpu: f32,
    pub ram: UsageData,
    pub swap: UsageData,
    pub disk: UsageData,
    pub network: NetData,
    pub temp: CPUTemp,
}

#[derive(Serialize, Default)]
pub struct UsageData {
    pub used: u64,
    pub total: u64,
    pub percent: f32,
}

#[derive(Serialize, Default, Debug)]
pub struct NetData {
    pub sent: u64,
    pub received: u64,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RequestTypes {
    Page {
        page: String,
    },
    Cmd {
        cmd: String,
        args: Option<Vec<String>>,
    },
    Token {
        token: String,
    },
}

#[derive(Serialize)]
pub struct ProcessData {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub ram: u64,
    pub status: &'static str,
}

#[derive(Serialize)]
pub struct ProcessList {
    pub processes: Vec<ProcessData>,
}

#[derive(Serialize, Default)]
pub struct DPSoftwareData {
    pub id: i16,
    pub name: String,
    pub description: String,
    pub dependencies: String,
    pub docs: String,
}

#[derive(Serialize, Default)]
pub struct DPSoftwareList {
    pub installed: Vec<DPSoftwareData>,
    pub uninstalled: Vec<DPSoftwareData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
}

#[derive(Serialize, Default)]
pub struct HostData {
    pub hostname: String,
    pub uptime: u64,
    pub arch: &'static str,
    pub kernel: String,
    pub dp_version: String,
    pub packages: usize,
    pub upgrades: u32,
    pub nic: String,
    pub ip: String,
}

#[derive(Serialize, Default)]
pub struct ServiceData {
    pub name: String,
    pub log: String,
    pub status: &'static str,
    pub start: String,
}

#[derive(Serialize)]
pub struct ServiceList {
    pub services: Vec<ServiceData>,
}

#[derive(Serialize)]
pub struct GlobalData {
    pub update: String,
    pub version: String,
    pub login: bool,
    pub update_check: bool,
    #[cfg(feature = "frontend")]
    pub nodes: Vec<String>,
    pub temp_unit: TempUnit,
}

#[derive(Serialize)]
pub struct BrowserData {
    pub path: String,
    pub name: String,
    pub subtype: String,
    pub maintype: String,
    pub prettytype: String,
    pub size: u64,
}

#[derive(Serialize, Default)]
pub struct BrowserList {
    pub contents: Vec<BrowserData>,
}

#[derive(Deserialize, Debug)]
pub struct FileRequest {
    #[serde(default)]
    pub cmd: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub arg: String,
}

#[derive(Serialize)]
pub struct FileSize {
    pub size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct JWTClaims {
    pub iss: String,
    pub exp: u64,
    pub iat: u64,
    pub fingerprint: String,
}

#[derive(Serialize, Default)]
pub struct CPUTemp {
    pub temp: Option<i16>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TempUnit {
    Fahrenheit,
    Celsius,
}

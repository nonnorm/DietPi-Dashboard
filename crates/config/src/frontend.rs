use std::path::PathBuf;

use anyhow::Result;
use log::LevelFilter;
use serde::Deserialize;
use toml_migrate::build_migration_chain;

use crate::custom_serde::HexArray;
use crate::generate_config_file;

pub type FrontendConfig = FrontendConfigV1;

pub fn get_config() -> Result<FrontendConfig> {
    crate::read_config("config-frontend.toml", generate_config_file)
}

fn generate_config_file(config: &FrontendConfig) -> String {
    generate_config_file!(
        "config-frontend.template.toml",
        http_port = config.http_port,
        backend_port = config.backend_port,
        log_level = config.log_level,
        enable_tls = config.enable_tls,
        key_path = config.key_path,
        cert_path = config.cert_path,
        enable_login = config.enable_login,
        hash = config.hash,
        secret = config.secret
    )
}

build_migration_chain!(FrontendConfigV0 = 0, FrontendConfigV1 = 1);

#[derive(Deserialize)]
pub struct FrontendConfigV1 {
    pub http_port: u16,
    pub backend_port: u16,
    pub log_level: LevelFilter,
    pub enable_tls: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub enable_login: bool,
    pub hash: String,
    pub secret: HexArray<32>,
}

impl Default for FrontendConfigV1 {
    fn default() -> Self {
        Self {
            http_port: 5252,
            backend_port: 5353,
            log_level: LevelFilter::Info,
            enable_tls: false,
            cert_path: PathBuf::new(),
            key_path: PathBuf::new(),
            enable_login: false,
            hash: String::new(),
            secret: HexArray(rand::random()),
        }
    }
}

impl From<FrontendConfigV0> for FrontendConfigV1 {
    fn from(val: FrontendConfigV0) -> Self {
        let default = Self::default();

        let secret = val
            .secret
            .and_then(|x| data_encoding::HEXLOWER.decode(x.as_bytes()).ok())
            .and_then(|x| x.try_into().ok())
            .map(HexArray);

        Self {
            http_port: val.port.unwrap_or(default.http_port),
            backend_port: default.backend_port,
            log_level: val.log_level.unwrap_or(default.log_level),
            enable_tls: val.tls.unwrap_or(default.enable_tls),
            cert_path: val.cert.unwrap_or(default.cert_path),
            key_path: val.key.unwrap_or(default.key_path),
            enable_login: val.pass.unwrap_or(default.enable_login),
            hash: val.hash.unwrap_or(default.hash),
            secret: secret.unwrap_or(default.secret),
        }
    }
}

// Taken from the original version of DietPi-Dashboard
#[derive(Deserialize)]
pub struct FrontendConfigV0 {
    pub port: Option<u16>,
    pub log_level: Option<LevelFilter>,
    pub tls: Option<bool>,
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub pass: Option<bool>,
    pub hash: Option<String>,
    pub secret: Option<String>,
}

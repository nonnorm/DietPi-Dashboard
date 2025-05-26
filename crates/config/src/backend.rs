use std::net::SocketAddr;

use anyhow::Result;
use log::LevelFilter;
use serde::Deserialize;
use toml_migrate::build_migration_chain;

use crate::custom_serde;
use crate::custom_serde::HexArray;
use crate::generate_config_file;

pub type BackendConfig = BackendConfigV1;

pub fn get_config() -> Result<BackendConfig> {
    crate::read_config("config-backend.toml", generate_config_file)
}

fn generate_config_file(config: &BackendConfig) -> String {
    generate_config_file!(
        "config-backend.template.toml",
        log_level = config.log_level,
        frontend_addr = config.frontend_addr,
        nickname = config.nickname,
        secret = config.secret,
        disks = config.disks
    )
}

build_migration_chain!(BackendConfigV0 = 0, BackendConfigV1 = 1);

#[derive(Deserialize)]
pub struct BackendConfigV1 {
    pub log_level: LevelFilter,
    pub frontend_addr: SocketAddr,
    pub nickname: String,
    pub secret: HexArray<32>,
    pub disks: Vec<String>,
}

impl Default for BackendConfigV1 {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
            frontend_addr: ([127, 0, 0, 1], 5353).into(),
            nickname: String::new(),
            secret: HexArray(rand::random()),
            disks: vec!["/".into()],
        }
    }
}

impl From<BackendConfigV0> for BackendConfigV1 {
    fn from(val: BackendConfigV0) -> Self {
        let default = Self::default();

        let secret = val
            .secret
            .and_then(|x| data_encoding::HEXLOWER.decode(x.as_bytes()).ok())
            .and_then(|x| x.try_into().ok())
            .map(HexArray);

        Self {
            log_level: val.log_level.unwrap_or(default.log_level),
            frontend_addr: default.frontend_addr,
            nickname: default.nickname,
            secret: secret.unwrap_or(default.secret),
            disks: default.disks,
        }
    }
}

// Taken from the original version of DietPi-Dashboard
#[derive(Deserialize)]
pub struct BackendConfigV0 {
    pub log_level: Option<LevelFilter>,
    pub secret: Option<String>,
}

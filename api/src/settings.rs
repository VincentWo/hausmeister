use std::collections::HashSet;

use config::{Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct DbConfig {
    pub(crate) url: String,
}

fn false_default() -> bool {
    false
}

#[derive(Debug, Deserialize)]
pub(crate) struct AppConfig {
    #[serde(default)]
    pub(crate) allowed_origins: HashSet<String>,
    #[serde(default = "false_default")]
    pub(crate) allow_localhost: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) database: DbConfig,
    pub(crate) app: AppConfig,
}

pub(crate) fn read_config() -> color_eyre::Result<Config> {
    let conf = config::Config::builder()
        .add_source(File::with_name("config.toml").required(false))
        .add_source(Environment::with_prefix("HM").separator("_"))
        .build()?;

    Ok(conf.try_deserialize()?)
}

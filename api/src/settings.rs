use config::{Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct DbConfig {
    pub(crate) url: String,
}
#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) database: DbConfig,
}

pub(crate) fn read_config() -> color_eyre::Result<Config> {
    let conf = config::Config::builder()
        .add_source(File::with_name("config.toml").required(false))
        .add_source(Environment::with_prefix("HM").separator("_"))
        .build()?;

    Ok(conf.try_deserialize()?)
}

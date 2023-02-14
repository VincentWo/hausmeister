//! Loading settings from files and environment
use std::collections::HashSet;

use config::{Environment, File};
use serde::Deserialize;

/// Config for PostgreSQL Connection
#[derive(Debug, Deserialize)]
pub(crate) struct DbConfig {
    /// The connection url for including password, username, db, etc.
    pub(crate) url: String,
}

/// General app config
#[derive(Debug, Deserialize)]
pub(crate) struct AppConfig {
    /// Used by CORS middleware, should include all origins the app
    /// is supposed to run on - does not support any kind of regex at
    /// the moment
    #[serde(default)]
    pub(crate) allowed_origins: HashSet<String>,
    /// If set all localhost origins are considered valid CORS origins
    /// Usefull for development, should be deactivated in production.
    #[serde(default = "false_default")]
    pub(crate) allow_localhost: bool,
}

/// Collection of all config areas
#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    /// Config affecting the database connection
    pub(crate) database: DbConfig,
    /// General application config
    pub(crate) app: AppConfig,
}

/// Reads config from config.toml + environment
///
/// For a documentation of possible values see the docs of
/// [Config], env values require a prefix of `HM` and nestings
/// are separated by `_`
pub(crate) fn read_config() -> color_eyre::Result<Config> {
    let conf = config::Config::builder()
        .add_source(File::with_name("config.toml").required(false))
        .add_source(Environment::with_prefix("HM").separator("_"))
        .build()?;

    Ok(conf.try_deserialize()?)
}

/// Proxy for serde default
///
/// This is needed since Serde only supports paths right now as default
/// values, see:
/// <https://github.com/serde-rs/serde/issues/368>
fn false_default() -> bool {
    false
}

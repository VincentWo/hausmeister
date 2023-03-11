//! Loading settings from files and environment
use std::{collections::HashSet, net::IpAddr, str::FromStr};

use color_eyre::eyre::Context;
use config::{Environment, File};
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, ConnectOptions};
use webauthn_rs::prelude::Url;

/// Config for PostgreSQL Connection
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DbConfig {
    /// Connect using a Database URL
    DatabaseUrl {
        /// The connection url for including password, username, db, etc.
        url: String,
    },
    /// Connect using different parameters
    Parameters {
        username: String,
        password: String,
        host: String,
        port: Option<u16>,
        db_name: Option<String>,
    },
}

impl TryInto<PgConnectOptions> for &DbConfig {
    type Error = <PgConnectOptions as FromStr>::Err;

    fn try_into(self) -> Result<PgConnectOptions, Self::Error> {
        match self {
            DbConfig::DatabaseUrl { url } => url.parse(),
            DbConfig::Parameters {
                username,
                password,
                host,
                port,
                db_name,
            } => {
                let mut options = PgConnectOptions::new()
                    .username(username)
                    .password(password)
                    .host(host);
                if let Some(port) = port {
                    options = options.port(*port);
                }
                if let Some(db_name) = db_name {
                    options = options.database(db_name);
                }

                options.log_statements(tracing::log::LevelFilter::Trace);
                Ok(options)
            }
        }
    }
}

fn default_port() -> u16 {
    3779
}

/// General app config
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// Used by CORS middleware, should include all origins the app
    /// is supposed to run on - does not support any kind of regex at
    /// the moment
    #[serde(default)]
    pub allowed_origins: HashSet<Url>,
    /// If set all localhost origins are considered valid CORS origins
    /// Usefull for development, should be deactivated in production.
    #[serde(default = "false_default")]
    pub allow_localhost: bool,

    /// The IP the server is listening on
    pub listen_on: IpAddr,

    /// The port the server is going to listen to
    #[serde(default = "default_port")]
    pub port: u16,
}

/// Configuring redis connection
#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    /// The URL
    pub url: Url,
}

/// Collection of all config areas
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Config affecting the database connection
    pub database: DbConfig,
    /// General application config
    pub app: AppConfig,
    /// Config for the redis connection
    pub redis: RedisConfig,
}

/// Reads config from config.toml + environment
///
/// For a documentation of possible values see the docs of
/// [Config], env values require a prefix of `HM` and nestings
/// are separated by `_`
pub fn read_config() -> color_eyre::Result<Config> {
    let conf = config::Config::builder()
        .add_source(File::with_name("config.toml").required(false))
        .add_source(
            Environment::with_prefix("HM")
                .separator("__")
                .prefix_separator("_"),
        )
        .build()
        .wrap_err("Reading the config")?;

    conf.try_deserialize().wrap_err("Parsing the config")
}

/// Proxy for serde default
///
/// This is needed since Serde only supports paths right now as default
/// values, see:
/// <https://github.com/serde-rs/serde/issues/368>
fn false_default() -> bool {
    false
}

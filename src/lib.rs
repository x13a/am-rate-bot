use serde::Deserialize;
use std::{env, fs, sync::Arc};

pub mod bot;
pub mod collector;
pub mod db;
pub mod gen;
pub mod graph;
pub mod source;

pub const DUNNO: &str = r"¯\_(ツ)_/¯";
const ENV_CONFIG: &str = "CONFIG";

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bot: Bot,
    pub src: source::Config,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Bot {
    pub reqwest_timeout: u64,
    pub update_interval: u64,
    pub polling: bool,
    pub webhook: Webhook,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Webhook {
    pub host: String,
    pub port: u16,
    pub cert: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Arc<Self>> {
        let cfg = toml::from_str(fs::read_to_string(env::var(ENV_CONFIG)?)?.as_str())?;
        Ok(Arc::new(cfg))
    }
}

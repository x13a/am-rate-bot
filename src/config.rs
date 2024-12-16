use crate::source;
use serde::Deserialize;
use std::{env, fs, sync::Arc};

const ENV_CONFIG: &str = "BOT_CONFIG";

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bot: Bot,
    pub gen: Gen,
    pub src: source::Config,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Bot {
    pub reqwest_timeout: u64,
    pub update_interval: u64,
    pub polling: bool,
    pub webhook: Webhook,

    pub welcome_msg: String,
    pub name: String,
    pub about: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Webhook {
    pub url: String,
    pub port: u16,
    pub cert: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Gen {
    pub rate_dp: u32,
    pub diff_dp: u32,
}

impl Config {
    pub fn load() -> anyhow::Result<Arc<Self>> {
        let cfg = toml::from_str(&fs::read_to_string(env::var(ENV_CONFIG)?)?)?;
        Ok(Arc::new(cfg))
    }
}

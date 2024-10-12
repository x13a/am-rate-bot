pub mod bot;
pub mod collector;
pub mod generator;
pub mod sources;

pub const DUNNO: &str = r"¯\_(ツ)_/¯";

#[derive(Debug, Copy, Clone)]
pub struct Opts {
    pub reqwest_timeout: u64,
    pub update_interval: u64,
}

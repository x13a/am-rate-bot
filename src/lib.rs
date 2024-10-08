use argh::FromArgs;

pub mod bot;
pub mod collector;
pub mod generator;
pub mod sources;

pub const DUNNO: &str = r"¯\_(ツ)_/¯";

#[derive(Debug, FromArgs, Copy, Clone)]
/// options:
pub struct Opts {
    /// reqwest timeout
    #[argh(option, default = "10")]
    pub timeout: u64,
    /// rates collect interval
    #[argh(option, default = "5 * 60")]
    pub update_interval: u64,
}

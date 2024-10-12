use crate::sources::lsoft::{request::Request, LSoftRequest};
pub use crate::sources::unibank::Response;
use crate::sources::SourceConfigTrait;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub req: Request,
    pub commission_rate: Decimal,
}

impl SourceConfigTrait for Config {
    fn rates_url(&self) -> String {
        self.rates_url.clone()
    }
}

impl LSoftRequest for Config {
    fn req(&self) -> Request {
        self.req.clone()
    }
}

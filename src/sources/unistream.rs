use crate::sources::lsoft::{request::Request, LSoftRequest};
pub use crate::sources::unibank::{get, Response};
use crate::sources::RatesConfigTrait;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub req: Request,
    pub commission_rate_from_bank: Decimal,
    pub commission_rate_from_any_card: Decimal,
}

impl RatesConfigTrait for Config {
    fn rates_url(&self) -> String {
        self.rates_url.clone()
    }
}

impl LSoftRequest for Config {
    fn req(&self) -> Request {
        self.req.clone()
    }
}

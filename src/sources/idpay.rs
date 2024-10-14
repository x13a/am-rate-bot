pub use crate::sources::idbank::Response;
use crate::sources::SourceConfigTrait;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub commission_rate: Decimal,
    pub commission_rate_to_ru_card: Decimal,
}

impl SourceConfigTrait for Config {
    fn rates_url(&self) -> String {
        self.rates_url.clone()
    }
}

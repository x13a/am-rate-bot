pub use crate::sources::SourceConfig as Config;
use crate::sources::{de, Currency as ModCurrency, JsonResponse};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub currencies: Currencies,
}

#[derive(Debug, Deserialize)]
pub struct Currencies {
    pub cash: Vec<Currency>,
    pub no_cash: Vec<Currency>,
}

#[derive(Debug, Deserialize)]
pub struct Currency {
    #[serde(rename = "type", deserialize_with = "de::currency")]
    pub curr_type: ModCurrency,
    pub buy: Decimal,
    pub sell: Decimal,
}

impl JsonResponse for Response {}

use crate::sources::utils::de_currency;
use crate::sources::{Currency, SourceSingleUrlTrait};
use rust_decimal::serde::arbitrary_precision_option;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://www.inecobank.am/api/rates";

#[derive(Debug, Deserialize)]
pub struct Response {
    pub success: bool,
    #[serde(default)]
    pub items: Option<Vec<Item>>,
}

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Item {
    #[serde(deserialize_with = "de_currency")]
    pub code: Currency,
    pub cash: Rate,
    pub cashless: Rate,
    pub online: Rate,
    pub cb: Rate,
    pub card: Rate,
}

#[derive(Debug, Deserialize)]
pub struct Rate {
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub buy: Option<Decimal>,
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")]
    pub sell: Option<Decimal>,
}

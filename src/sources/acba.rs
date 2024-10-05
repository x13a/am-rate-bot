use crate::sources::utils::de_currency;
use crate::sources::{Currency as SourcesCurrency, SourceSingleUrlTrait};
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://www.acbadigital.am/api/en/v2/rates";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub description: Option<String>,
    pub result_code: i32,
    #[serde(default)]
    pub result: Option<Result>,
    pub result_code_description: String,
}

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub rates: Rates,
}

#[derive(Debug, Deserialize)]
pub struct Rates {
    pub last_update_date: String,
    pub cash: Vec<Rate>,
    pub non_cash: Vec<Rate>,
    pub card: Vec<Rate>,
    pub cross: Vec<CrossRate>,
    pub currencies: Vec<Currency>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rate {
    pub buy: Decimal,
    pub sell: Decimal,
    #[serde(rename = "CB")]
    pub cb: String,
    #[serde(deserialize_with = "de_currency")]
    pub currency: SourcesCurrency,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CrossRate {
    pub buy: Decimal,
    pub sell: Decimal,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Currency {
    pub key: String,
    pub value: String,
}

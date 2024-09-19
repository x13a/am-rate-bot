use crate::sources::utils::{de_currency, de_f64};
use crate::sources::{Currency as SourcesCurrency, SourceSingleUrlTrait};
use serde::Deserialize;

const API_URL: &str = "https://www.acbadigital.am/api/en/v2/rates";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub description: Option<String>,
    pub result_code: u8,
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
    #[serde(deserialize_with = "de_f64")]
    pub buy: f64,
    #[serde(deserialize_with = "de_f64")]
    pub sell: f64,
    #[serde(deserialize_with = "de_f64", rename = "CB")]
    pub cb: f64,
    #[serde(deserialize_with = "de_currency")]
    pub currency: SourcesCurrency,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CrossRate {
    #[serde(deserialize_with = "de_f64")]
    pub buy: f64,
    #[serde(deserialize_with = "de_f64")]
    pub sell: f64,
    #[serde(deserialize_with = "de_currency")]
    pub currency: SourcesCurrency,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Currency {
    pub key: String,
    pub value: String,
}

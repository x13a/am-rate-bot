use crate::sources::utils::de_currency;
use crate::sources::{Currency, RateType, SourceCashUrlTrait};
use rust_decimal::serde::arbitrary_precision;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str =
    "https://mobileapi.fcc.am/FCBank.Mobile.Api_V2/api/publicInfo/getRates?langID=2&payType";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    #[serde(default)]
    pub rates: Option<Vec<Rate>>,
    pub result_code: i32,
    pub result_message: String,
}

impl SourceCashUrlTrait for Response {
    fn url_cash() -> String {
        format!("{API_URL}={}", RateType::Cash as u8)
    }

    fn url_no_cash() -> String {
        format!("{API_URL}={}", RateType::NoCash as u8)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rate {
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub buy: Decimal,
    #[serde(deserialize_with = "de_currency")]
    pub id: Currency,
    pub pay_type: u8,
    pub prev_buy: f64,
    pub prev_sale: f64,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")]
    pub sale: Decimal,
    pub sort_id: u64,
    pub unit: f32,
}

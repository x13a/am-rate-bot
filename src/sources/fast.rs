use crate::sources::utils::de_currency;
use crate::sources::{Currency, RateType, SourceCashUrlTrait};
use const_format::formatcp;
use serde::Deserialize;

const API_URL: &str =
    "https://mobileapi.fcc.am/FCBank.Mobile.Api_V2/api/publicInfo/getRates?langID=2&payType";
const API_NO_CASH_URL: &str = formatcp!("{API_URL}={}", RateType::NoCash as u8);
const API_CASH_URL: &str = formatcp!("{API_URL}={}", RateType::Cash as u8);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    #[serde(default)]
    pub rates: Option<Vec<Rate>>,
    pub result_code: u16,
    pub result_message: String,
}

impl SourceCashUrlTrait for Response {
    fn url_cash() -> String {
        API_CASH_URL.into()
    }

    fn url_no_cash() -> String {
        API_NO_CASH_URL.into()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Rate {
    pub buy: f64,
    #[serde(deserialize_with = "de_currency")]
    pub id: Currency,
    pub pay_type: u8,
    pub prev_buy: f64,
    pub prev_sale: f64,
    pub sale: f64,
    pub sort_id: u8,
    pub unit: f32,
}

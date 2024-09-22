use crate::sources::{ExchangeRate, RateType, SourceCashUrlTrait};
use serde::Deserialize;

const API_URL: &str = "https://online.ameriabank.am/InternetBank/Api/exchangeRates";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase", transparent)]
pub struct Response {
    pub array_of_exchange_rate: Vec<ExchangeRate>,
}

impl SourceCashUrlTrait for Response {
    fn url_cash() -> String {
        format!("{API_URL}/{}", RateType::Cash as u8)
    }

    fn url_no_cash() -> String {
        format!("{API_URL}/{}", RateType::NoCash as u8)
    }
}

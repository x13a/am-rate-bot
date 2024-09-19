use crate::sources::{ExchangeRate, RateType, SourceCashUrlTrait};
use const_format::formatcp;
use serde::Deserialize;

const API_URL: &str = "https://online.ameriabank.am/InternetBank/Api/exchangeRates";
const API_NO_CASH_URL: &str = formatcp!("{API_URL}/{}", RateType::NoCash as u8);
const API_CASH_URL: &str = formatcp!("{API_URL}/{}", RateType::Cash as u8);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase", transparent)]
pub struct Response {
    pub array_of_exchange_rate: Vec<ExchangeRate>,
}

impl SourceCashUrlTrait for Response {
    fn url_cash() -> String {
        API_CASH_URL.into()
    }

    fn url_no_cash() -> String {
        API_NO_CASH_URL.into()
    }
}

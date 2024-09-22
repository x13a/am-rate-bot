use crate::sources::{RateType, SourceCashUrlTrait};

pub const API_URL: &str = "https://online.byblosbankarmenia.am/InternetBank/api/exchangeRates";

pub struct Response;

impl SourceCashUrlTrait for Response {
    fn url_cash() -> String {
        format!("{API_URL}/{}", RateType::Cash as u8)
    }

    fn url_no_cash() -> String {
        format!("{API_URL}/{}", RateType::NoCash as u8)
    }
}

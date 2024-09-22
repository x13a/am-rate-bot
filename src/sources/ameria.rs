use crate::sources::{RateType, SourceCashUrlTrait};

pub const API_URL: &str = "https://online.ameriabank.am/InternetBank/Api/exchangeRates";

pub struct Response;

impl SourceCashUrlTrait for Response {
    fn url_cash() -> String {
        format!("{API_URL}/{}", RateType::Cash as u8)
    }

    fn url_no_cash() -> String {
        format!("{API_URL}/{}", RateType::NoCash as u8)
    }
}

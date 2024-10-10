use crate::sources::{RateType, RateTypeResponse};

pub const API_URL: &str = "https://internetbank.amiobank.am/InternetBank/api/exchangeRates";

pub struct Response;

impl RateTypeResponse for Response {
    fn url_cash() -> String {
        format!("{API_URL}/{}", RateType::Cash as u8)
    }

    fn url_no_cash() -> String {
        format!("{API_URL}/{}", RateType::NoCash as u8)
    }
}

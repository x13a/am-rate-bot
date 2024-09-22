use crate::sources::utils::de_currency;
use crate::sources::Currency;
use crate::sources::Error;
use serde::Deserialize;

pub const API_URL: &str = "https://api.cba.am/exchangerates.asmx";
pub const SOAP12_EXCHANGE_RATES_LATEST: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope">
  <soap12:Body>
    <ExchangeRatesLatest xmlns="http://www.cba.am/" />
  </soap12:Body>
</soap12:Envelope>"#;

#[derive(Deserialize, Debug)]
#[serde(rename = "SoapEnvelope")]
pub struct Response {
    #[serde(rename = "@xmlns:soap")]
    pub xmlns_soap: String,
    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: String,
    #[serde(rename = "@xmlns:xsd")]
    pub xmlns_xsd: String,
    #[serde(rename = "Body")]
    pub soap_body: SoapBody,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SoapBody {
    pub exchange_rates_latest_response: ExchangeRatesLatestResponse,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRatesLatestResponse {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
    pub exchange_rates_latest_result: ExchangeRatesLatestResult,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRatesLatestResult {
    pub current_date: String,
    pub next_available_date: NextAvailableDate,
    pub previous_available_date: String,
    pub rates: Rates,
}

#[derive(Deserialize, Debug)]
pub struct NextAvailableDate {
    #[serde(rename = "@nil")]
    pub xsi_nil: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Rates {
    pub exchange_rate: Vec<ExchangeRate>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRate {
    #[serde(rename = "ISO", deserialize_with = "de_currency")]
    pub iso: Currency,
    pub amount: u16,
    pub rate: f64,
    pub difference: f64,
}

impl Response {
    pub fn url() -> String {
        API_URL.into()
    }

    pub async fn get_rates(c: &reqwest::Client) -> Result<Self, Error> {
        let body = c
            .post(Self::url())
            .header("Content-Type", "application/soap+xml; charset=utf-8")
            .body(SOAP12_EXCHANGE_RATES_LATEST)
            .send()
            .await?
            .text()
            .await?;
        let resp = quick_xml::de::from_str(&body)?;
        Ok(resp)
    }
}

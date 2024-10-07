use crate::sources::utils::{de_currency, de_decimal};
use crate::sources::Currency;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://api.cba.am/exchangerates.asmx";

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
    pub amount: u32,
    #[serde(deserialize_with = "de_decimal")]
    pub rate: Decimal,
    pub difference: String,
}

pub mod request {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename = "soap12:Envelope")]
    pub struct Soap12Envelope {
        #[serde(rename = "@xmlns:xsi")]
        pub xmlns_xsi: String,
        #[serde(rename = "@xmlns:xsd")]
        pub xmlns_xsd: String,
        #[serde(rename = "@xmlns:soap12")]
        pub xmlns_soap12: String,
        #[serde(rename = "soap12:Body")]
        pub soap12_body: Soap12Body,
    }

    #[derive(Serialize)]
    pub struct Soap12Body {
        #[serde(rename = "ExchangeRatesLatest")]
        pub exchange_rates_latest: ExchangeRatesLatest,
    }

    #[derive(Serialize)]
    pub struct ExchangeRatesLatest {
        #[serde(rename = "@xmlns")]
        pub xmlns: String,
    }
}

impl Response {
    pub fn url() -> String {
        API_URL.into()
    }

    pub async fn get_rates(c: &reqwest::Client) -> anyhow::Result<Self> {
        let req_body = request::Soap12Envelope {
            xmlns_xsi: "http://www.w3.org/2001/XMLSchema-instance".into(),
            xmlns_xsd: "http://www.w3.org/2001/XMLSchema".into(),
            xmlns_soap12: "http://www.w3.org/2003/05/soap-envelope".into(),
            soap12_body: request::Soap12Body {
                exchange_rates_latest: request::ExchangeRatesLatest {
                    xmlns: "http://www.cba.am/".into(),
                },
            },
        };
        let xml = c
            .post(Self::url())
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/soap+xml; charset=utf-8",
            )
            .body(quick_xml::se::to_string(&req_body).expect("xml serialization failed"))
            .send()
            .await?
            .text()
            .await?;
        let resp = quick_xml::de::from_str(&xml)?;
        Ok(resp)
    }
}

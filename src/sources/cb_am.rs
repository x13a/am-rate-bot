pub use crate::sources::RatesConfig as Config;
use crate::sources::{de, Currency, RatesConfigTrait};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename = "SoapEnvelope")]
pub struct Response {
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
    pub exchange_rates_latest_result: ExchangeRatesLatestResult,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRatesLatestResult {
    pub rates: Rates,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Rates {
    pub exchange_rate: Vec<ExchangeRate>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRate {
    #[serde(rename = "ISO", deserialize_with = "de::currency")]
    pub iso: Currency,
    #[serde(deserialize_with = "de::decimal")]
    pub rate: Decimal,
    #[serde(deserialize_with = "de::decimal")]
    pub amount: Decimal,
}

pub mod request {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename = "soap12:Envelope")]
    pub struct Request {
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

pub async fn get<T>(client: &reqwest::Client, config: &T) -> anyhow::Result<Response>
where
    T: RatesConfigTrait,
{
    let req_data = request::Request {
        xmlns_xsi: "http://www.w3.org/2001/XMLSchema-instance".into(),
        xmlns_xsd: "http://www.w3.org/2001/XMLSchema".into(),
        xmlns_soap12: "http://www.w3.org/2003/05/soap-envelope".into(),
        soap12_body: request::Soap12Body {
            exchange_rates_latest: request::ExchangeRatesLatest {
                xmlns: "http://www.cba.am/".into(),
            },
        },
    };
    let xml = client
        .post(config.rates_url())
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/soap+xml; charset=utf-8",
        )
        .body(quick_xml::se::to_string(&req_data)?)
        .send()
        .await?
        .text()
        .await?;
    let resp = quick_xml::de::from_str(&xml)?;
    Ok(resp)
}

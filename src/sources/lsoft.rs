use crate::sources::utils::{de_currency, de_empty_decimal};
use crate::sources::Currency;
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub const APHENA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<request client="mobile" device="android" handler="aphena" lang="2" operation="getCurrencyList">
   <accesstoken></accesstoken>
   <id>5</id>
   <getCurrencyListParameters>
      <currency></currency>
   </getCurrencyListParameters>
   <userid></userid>
</request>"#;

pub trait SourceAphenaTrait {
    fn url() -> String;

    async fn get_rates<T>(c: &reqwest::Client) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let body = c
            .post(Self::url())
            .body(APHENA)
            .send()
            .await?
            .text()
            .await?;
        let resp: T = quick_xml::de::from_str(&body)?;
        Ok(resp)
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "@operation")]
    pub operation: String,
    #[serde(rename = "@handler")]
    pub handler: String,
    pub parentid: u64,
    #[serde(rename = "Id")]
    pub id: u64,
    #[serde(rename = "getCurrencyList")]
    pub get_currency_list: GetCurrencyList,
}

#[derive(Debug, Deserialize)]
pub struct GetCurrencyList {
    #[serde(rename = "CurrencyList", default)]
    pub currency_list: Option<Vec<CurrencyList>>,
    #[serde(rename = "errorCode")]
    pub error_code: i32,
}

#[derive(Debug, Deserialize)]
pub struct CurrencyList {
    #[serde(rename = "externalId", deserialize_with = "de_currency")]
    pub external_id: Currency,
    pub cb: f64,
    #[serde(deserialize_with = "de_empty_decimal")]
    pub sell: Option<Decimal>,
    #[serde(deserialize_with = "de_empty_decimal")]
    pub buy: Option<Decimal>,
    pub trf30: Trf30,
    pub trf31: Trf31,
    #[serde(rename = "CshSell", deserialize_with = "de_empty_decimal")]
    pub csh_sell: Option<Decimal>,
    #[serde(rename = "CshBuy", deserialize_with = "de_empty_decimal")]
    pub csh_buy: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct Trf30;

#[derive(Debug, Deserialize)]
pub struct Trf31;

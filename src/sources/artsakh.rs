use crate::sources::utils::{de_currency, de_option_f64};
use crate::sources::{Currency, Error};
use serde::Deserialize;

const API_URL: &str = "https://m.artsakhbank.am:9443/get_ART.php";
const GET_CURRENCY_LIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<request client="mobile" device="android" handler="aphena" lang="2" operation="getCurrencyList" appversion="2.7.5">
   <accesstoken></accesstoken>
   <id>6</id>
   <getCurrencyListParameters>
      <currency></currency>
   </getCurrencyListParameters>
   <userid></userid>
</request>"#;

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
    pub error_code: u8,
}

#[derive(Debug, Deserialize)]
pub struct CurrencyList {
    #[serde(rename = "externalId", deserialize_with = "de_currency")]
    pub external_id: Currency,
    pub cb: f64,
    #[serde(deserialize_with = "de_option_f64")]
    pub sell: Option<f64>,
    #[serde(deserialize_with = "de_option_f64")]
    pub buy: Option<f64>,
    pub trf30: Trf30,
    pub trf31: Trf31,
    #[serde(rename = "CshSell")]
    pub csh_sell: f64,
    #[serde(rename = "CshBuy")]
    pub csh_buy: f64,
}

#[derive(Debug, Deserialize)]
pub struct Trf30 {}

#[derive(Debug, Deserialize)]
pub struct Trf31 {}

impl Response {
    pub fn url() -> String {
        API_URL.into()
    }

    pub async fn get_rates(c: &reqwest::Client) -> Result<Self, Error> {
        let body = c
            .post(Self::url())
            .body(GET_CURRENCY_LIST)
            .send()
            .await?
            .text()
            .await?;
        let resp = quick_xml::de::from_str(&body)?;
        Ok(resp)
    }
}

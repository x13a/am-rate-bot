use crate::sources::utils::{de_currency, de_option_f64};
use crate::sources::{Currency, Error, SourceSingleUrlTrait};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub const API_URL: &str = "https://www.idbanking.am/api/MyInfo/getCurrencyRateMobile";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub op_code: i32,
    pub op_desc: String,
    #[serde(default)]
    pub result: Option<Result>,
    pub service_id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Result {
    pub currency_rate: Vec<CurrencyRate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CurrencyRate {
    #[serde(deserialize_with = "de_option_f64")]
    pub buy: Option<f64>,
    pub cards_buy: String,
    pub cards_sell: String,
    pub cb: String,
    pub country: String,
    #[serde(deserialize_with = "de_option_f64")]
    pub csh_buy: Option<f64>,
    #[serde(deserialize_with = "de_option_f64")]
    pub csh_buy_trf: Option<f64>,
    #[serde(deserialize_with = "de_option_f64")]
    pub csh_sell: Option<f64>,
    #[serde(deserialize_with = "de_option_f64")]
    pub csh_sell_trf: Option<f64>,
    pub external_id: String,
    pub iso_code: String,
    #[serde(deserialize_with = "de_currency")]
    pub iso_txt: Currency,
    pub loan: String,
    pub name: Name,
    #[serde(deserialize_with = "de_option_f64")]
    pub sell: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Name {
    pub translation: Vec<Translation>,
}

#[derive(Debug, Deserialize)]
pub struct Translation {
    pub lang: String,
    pub value: String,
}

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }

    async fn get_rates<T>(c: &Client) -> std::result::Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let resp = c
            .post(Self::url())
            .header(reqwest::header::CONTENT_LENGTH, 0)
            .send()
            .await?
            .json::<T>()
            .await?;
        Ok(resp)
    }
}

use crate::sources::utils::{de_currency, de_f64, de_option_f64};
use crate::sources::{Currency, Error, SourceSingleUrlTrait};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{de, Deserialize, Deserializer};

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
    #[serde(deserialize_with = "de_option_f64")]
    pub cards_buy: Option<f64>,
    #[serde(deserialize_with = "de_option_f64")]
    pub cards_sell: Option<f64>,
    #[serde(deserialize_with = "de_f64")]
    pub cb: f64,
    pub country: String,
    #[serde(deserialize_with = "de_option_f64")]
    pub csh_buy: Option<f64>,
    #[serde(deserialize_with = "de_option_f64")]
    pub csh_buy_trf: Option<f64>,
    #[serde(deserialize_with = "de_option_f64")]
    pub csh_sell: Option<f64>,
    #[serde(deserialize_with = "de_option_f64")]
    pub csh_sell_trf: Option<f64>,
    #[serde(deserialize_with = "de_currency")]
    pub external_id: Currency,
    #[serde(deserialize_with = "de_u32")]
    pub iso_code: u32,
    #[serde(deserialize_with = "de_currency")]
    pub iso_txt: Currency,
    #[serde(deserialize_with = "de_f64")]
    pub loan: f64,
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

fn de_u32<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let u = s.parse::<u32>().map_err(de::Error::custom)?;
    Ok(u)
}

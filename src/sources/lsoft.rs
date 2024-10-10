use crate::sources::{de_currency, de_empty_decimal, Currency};
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub mod request {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename = "request")]
    pub struct Request {
        #[serde(rename = "@client")]
        pub client: String,
        #[serde(rename = "@device")]
        pub device: String,
        #[serde(rename = "@handler")]
        pub handler: String,
        #[serde(rename = "@lang")]
        pub lang: String,
        #[serde(rename = "@operation")]
        pub operation: String,
        pub accesstoken: String,
        pub id: String,
        #[serde(rename = "getCurrencyListParameters")]
        pub get_currency_list_parameters: GetCurrencyListParameters,
        pub userid: String,
    }

    #[derive(Serialize)]
    pub struct GetCurrencyListParameters {
        pub currency: String,
    }
}

pub trait AphenaResponse {
    fn url() -> String;

    #[allow(async_fn_in_trait)]
    async fn get_rates<T>(c: &reqwest::Client) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let req_body = request::Request {
            client: "mobile".into(),
            device: "android".into(),
            handler: "aphena".into(),
            lang: "2".into(),
            operation: "getCurrencyList".into(),
            accesstoken: "".into(),
            id: "5".into(),
            get_currency_list_parameters: request::GetCurrencyListParameters {
                currency: "".into(),
            },
            userid: "".into(),
        };
        let body = c
            .post(Self::url())
            .body(quick_xml::se::to_string(&req_body).expect("xml serialization failed"))
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

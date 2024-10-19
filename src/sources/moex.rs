use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use std::env;

pub const ENV_TINKOFF_TOKEN: &str = "TINKOFF_TOKEN";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderBookResponse {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quotation {
    pub units: String,
    pub nano: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub price: Quotation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct GetOrderBookRequest {
    pub depth: i32,
    pub instrument_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyResponse {
    pub instrument: Currency,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub nominal: MoneyValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyValue {
    pub units: String,
    pub nano: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct InstrumentRequest {
    pub id_type: InstrumentIdType,
    pub class_code: String,
    pub id: String,
}

#[derive(Debug, Serialize_repr)]
#[repr(u8)]
pub enum InstrumentIdType {
    InstrumentIdUnspecified = 0,
    InstrumentIdTypeFigi,
    InstrumentIdTypeTicker,
    InstrumentIdTypeUid,
    InstrumentIdTypePositionUid,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub base_url: String,
    pub path_order_book: String,
    pub path_currency: String,
    pub enabled: bool,
    pub req: GetOrderBookRequest,
}

pub async fn get_order_book(
    client: &reqwest::Client,
    config: &Config,
) -> anyhow::Result<GetOrderBookResponse> {
    let req_data = GetOrderBookRequest {
        instrument_id: config.req.instrument_id.clone(),
        depth: config.req.depth,
    };
    _get(client, &config.base_url, &config.path_order_book, &req_data).await
}

pub async fn get_currency(
    client: &reqwest::Client,
    config: &Config,
) -> anyhow::Result<CurrencyResponse> {
    let req_data = InstrumentRequest {
        id_type: InstrumentIdType::InstrumentIdTypeFigi,
        class_code: "CETS".into(),
        id: config.req.instrument_id.clone(),
    };
    _get(client, &config.base_url, &config.path_currency, &req_data).await
}

async fn _get<T1, T2>(
    client: &reqwest::Client,
    base_url: &String,
    url_path: &String,
    req_data: &T2,
) -> anyhow::Result<T1>
where
    T1: DeserializeOwned,
    T2: Serialize + ?Sized,
{
    let token = env::var(ENV_TINKOFF_TOKEN)?;
    let resp = client
        .post(format!("{}/{}", base_url, url_path))
        .json(req_data)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await?
        .json()
        .await?;
    Ok(resp)
}

use serde::{Deserialize, Serialize};
use std::env;

pub const ENV_TINKOFF_TOKEN: &str = "TINKOFF_TOKEN";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

#[derive(Debug, Deserialize)]
pub struct Price {
    pub units: String,
    pub nano: i32,
}

#[derive(Debug, Deserialize)]
pub struct Order {
    pub price: Price,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Request {
    pub figi: String,
    pub depth: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub req: Request,
}

impl Response {
    pub async fn get_rates(client: &reqwest::Client, config: &Config) -> anyhow::Result<Self> {
        let token = env::var(ENV_TINKOFF_TOKEN)?;
        let req_data = Request {
            figi: config.req.figi.clone(),
            depth: config.req.depth,
        };
        let resp = client
            .post(config.rates_url.clone())
            .json(&req_data)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }
}

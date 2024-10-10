use serde::Deserialize;
use std::env;

pub const ENV_TINKOFF_TOKEN: &str = "TINKOFF_TOKEN";
pub const API_URL: &str = "https://invest-public-api.tinkoff.ru/rest/tinkoff.public.invest.api.contract.v1.MarketDataService/GetOrderBook";
pub const AMDRUB_TOM_FIGI: &str = "BBG0013J7V24";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub figi: String,
    pub depth: i32,
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub last_price: Price,
    pub close_price: Price,
    pub limit_up: Price,
    pub limit_down: Price,
    pub instrument_uid: String,
    pub last_price_ts: String,
    pub close_price_ts: String,
}

#[derive(Debug, Deserialize)]
pub struct Price {
    pub units: String,
    pub nano: i32,
}

#[derive(Debug, Deserialize)]
pub struct Order {
    pub quantity: String,
    pub price: Price,
}

pub mod request {
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Body {
        pub figi: String,
        pub instrument_id: String,
        pub depth: i32,
    }
}

impl Response {
    pub fn url() -> String {
        API_URL.into()
    }

    pub async fn get_rates(c: &reqwest::Client) -> anyhow::Result<Self> {
        let token = env::var(ENV_TINKOFF_TOKEN)?;
        let req_body = request::Body {
            figi: AMDRUB_TOM_FIGI.into(),
            instrument_id: "".into(),
            depth: 1,
        };
        let resp = c
            .post(Self::url())
            .json(&req_body)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }
}

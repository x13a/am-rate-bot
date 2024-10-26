use crate::source::{Currency, Rate, RateType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub convert_rate: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct Request {
    pub country_code_from: String,
    pub country_id_from: u64,
    pub country_code_to: String,
    pub country_id_to: u64,
    pub currency_id_from: u64,
    pub currency_id_to: u64,
    pub summ_send: u64,
    pub direction: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub req: Request,
}

async fn post(client: &reqwest::Client, config: &Config) -> anyhow::Result<Response> {
    let req_data = Request {
        country_code_from: config.req.country_code_from.clone(),
        country_id_from: config.req.country_id_from,
        country_code_to: config.req.country_code_to.clone(),
        country_id_to: config.req.country_id_to,
        currency_id_from: config.req.currency_id_from,
        currency_id_to: config.req.currency_id_to,
        summ_send: config.req.summ_send,
        direction: config.req.direction.clone(),
    };
    let mut resp = client
        .post(config.rates_url.clone())
        .form(&req_data)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    const CLOSE_SCRIPT_TAG: &str = "</script>";
    if let Some(idx) = resp.find(CLOSE_SCRIPT_TAG) {
        resp.drain(..idx + CLOSE_SCRIPT_TAG.len());
    }
    let resp = serde_json::from_str(&resp.trim())?;
    Ok(resp)
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = post(client, config).await?;
    Ok(vec![Rate {
        from: Currency::rub(),
        to: Currency::default(),
        rate_type: RateType::NoCash,
        buy: Some(resp.convert_rate),
        sell: None,
    }])
}

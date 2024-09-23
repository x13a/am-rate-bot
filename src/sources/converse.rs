use crate::sources::utils::{de_currency, de_rate_type};
use crate::sources::{Currency as SourceCurrency, RateType, SourceSingleUrlTrait};
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

pub const API_URL: &str = "https://sapi.conversebank.am/api/v2/currencyrates";

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "Non Cash")]
    pub non_cash: Vec<Item>,
    #[serde(rename = "Card")]
    pub card: Vec<Item>,
    #[serde(rename = "Metal")]
    pub metal: Vec<Item>,
    #[serde(rename = "Cash")]
    pub cash: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub buy: f64,
    pub buy_diff: f64,
    pub sell: f64,
    pub sell_diff: f64,
    pub rate_date: String,
    #[serde(rename = "type", deserialize_with = "de_rate_type")]
    pub rate_type: RateType,
    #[serde(deserialize_with = "de_currency")]
    pub iso2: SourceCurrency,
    pub created_at: String,
    pub updated_at: String,
    pub currency: Currency,
}

#[derive(Debug, Deserialize)]
pub struct Currency {
    pub id: u64,
    #[serde(deserialize_with = "de_currency")]
    pub iso: SourceCurrency,
    pub position: usize,
    pub sign: Option<String>,
    pub use_for_loand: u8,
    pub use_for_deposites: u8,
    pub use_for_rates: u8,
    #[serde(deserialize_with = "de_i32")]
    pub status: i32,
    pub created_at: String,
    pub updated_at: String,
}

fn de_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let rv = match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse().map_err(de::Error::custom)?,
        Value::Number(n) => n.as_i64().ok_or(de::Error::custom(""))? as i32,
        _ => return Err(de::Error::custom("")),
    };
    Ok(rv)
}

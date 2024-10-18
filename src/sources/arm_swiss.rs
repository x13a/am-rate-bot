use crate::sources::JsonResponse;
pub use crate::sources::SourceConfig as Config;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub lmasbrate: Vec<LmasbRate>,
}

#[derive(Debug, Deserialize)]
pub struct LmasbRate {
    #[serde(rename = "ISO")]
    pub iso: String,
    #[serde(rename = "BID")]
    pub bid: Decimal,
    #[serde(rename = "OFFER")]
    pub offer: Decimal,
    #[serde(rename = "BID_cash")]
    pub bid_cash: Decimal,
    #[serde(rename = "OFFER_cash")]
    pub offer_cash: Decimal,
}

impl JsonResponse for Response {}

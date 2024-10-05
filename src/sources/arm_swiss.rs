use crate::sources::utils::de_currency;
use crate::sources::{Currency, SourceSingleUrlTrait};
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://www.armswissbank.am/include/ajax.php";

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Response {
    pub lmasbrate: Option<Vec<LmasbRate>>,
    pub imworldcuratesusd: Option<Vec<ImWorldCuRatesUSD>>,
    pub imcuioil: Option<Vec<ImCui>>,
    pub imcuigold: Option<Vec<ImCui>>,
    pub imwmius: Option<Vec<ImWmi>>,
    pub imwmiasiapac: Option<Vec<ImWmi>>,
    pub imwmieurus: Option<Vec<ImWmi>>,
    pub imlibor: Option<Vec<ImlIbor>>,
    pub imeuribor: Option<Vec<ImEurIbor>>,
    #[serde(rename = "lmgoldRate")]
    pub lmgold_rate: Option<Vec<LmGoldRate>>,
    pub imusakey: Option<Vec<ImKey>>,
    pub imeukey: Option<Vec<ImKey>>,
}

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}

#[derive(Debug, Deserialize)]
pub struct LmasbRate {
    #[serde(rename = "ISO", deserialize_with = "de_currency")]
    pub iso: Currency,
    #[serde(rename = "CURRENCY")]
    pub currency: String,
    #[serde(rename = "BID")]
    pub bid: Decimal,
    #[serde(rename = "OFFER")]
    pub offer: Decimal,
    pub inserttime: String,
    pub hert: String,
    #[serde(rename = "BID_cash")]
    pub bid_cash: Decimal,
    #[serde(rename = "OFFER_cash")]
    pub offer_cash: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct ImWorldCuRatesUSD {
    #[serde(rename = "ISO")]
    pub iso: String,
    #[serde(rename = "CURRENCY")]
    pub currency: String,
    #[serde(rename = "BIDRATE")]
    pub bidrate: String,
    #[serde(rename = "Change")]
    pub change: String,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct ImCui {
    pub index: String,
    #[serde(rename = "Price")]
    pub price: String,
    #[serde(rename = "NetChange")]
    pub net_change: String,
    #[serde(rename = "PctChange")]
    pub pct_change: String,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct ImWmi {
    pub index: String,
    #[serde(rename = "VALUE")]
    pub value: String,
    #[serde(rename = "NetChange")]
    pub net_change: String,
    #[serde(rename = "PctChange")]
    pub pct_change: String,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct ImlIbor {
    #[serde(rename = "LIBOR")]
    pub libor: String,
    #[serde(rename = "USD")]
    pub usd: String,
    #[serde(rename = "Change1")]
    pub change1: String,
    #[serde(rename = "EUR")]
    pub eur: String,
    #[serde(rename = "Change2")]
    pub change2: String,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct ImEurIbor {
    #[serde(rename = "EURIBOR")]
    pub euribor: String,
    #[serde(rename = "EUR")]
    pub eur: String,
    #[serde(rename = "Change")]
    pub change: String,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct LmGoldRate {
    pub name: String,
    pub weight: String,
    pub offer: String,
    pub hert: String,
    pub inserttime: String,
}

#[derive(Debug, Deserialize)]
pub struct ImKey {
    #[serde(rename = "KEY")]
    pub key: String,
    #[serde(rename = "CURRENT")]
    pub current: String,
    pub inserttime: String,
    pub hert: String,
}

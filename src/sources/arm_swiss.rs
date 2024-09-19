use crate::sources::utils::{de_currency, de_f64};
use crate::sources::{Currency, SourceSingleUrlTrait};
use serde::Deserialize;

const API_URL: &str = "https://www.armswissbank.am/include/ajax.php";

#[derive(Debug, Deserialize)]
pub struct Response {
    pub lmasbrate: Vec<LmasbRate>,
    // pub imworldcuratesusd: Vec<ImWorldCuRatesUSD>,
    // pub imcuioil: Vec<ImCui>,
    // pub imcuigold: Vec<ImCui>,
    // pub imwmius: Vec<ImWmi>,
    // pub imwmiasiapac: Vec<ImWmi>,
    // pub imwmieurus: Vec<ImWmi>,
    // pub imlibor: Vec<ImlIbor>,
    // pub imeuribor: Vec<ImEurIbor>,
    // #[serde(rename = "lmgoldRate")]
    // pub lmgold_rate: Vec<LmGoldRate>,
    // pub imusakey: Vec<ImKey>,
    // pub imeukey: Vec<ImKey>,
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
    #[serde(rename = "BID", deserialize_with = "de_f64")]
    pub bid: f64,
    #[serde(rename = "OFFER", deserialize_with = "de_f64")]
    pub offer: f64,
    pub inserttime: String,
    pub hert: String,
    #[serde(rename = "BID_cash", deserialize_with = "de_f64")]
    pub bid_cash: f64,
    #[serde(rename = "OFFER_cash", deserialize_with = "de_f64")]
    pub offer_cash: f64,
}

#[derive(Debug, Deserialize)]
pub struct ImWorldCuRatesUSD {
    #[serde(rename = "ISO", deserialize_with = "de_currency")]
    pub iso: Currency,
    #[serde(rename = "CURRENCY")]
    pub currency: String,
    #[serde(rename = "BIDRATE", deserialize_with = "de_f64")]
    pub bidrate: f64,
    #[serde(rename = "Change")]
    pub change: String,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct ImCui {
    pub index: String,
    #[serde(rename = "Price", deserialize_with = "de_f64")]
    pub price: f64,
    #[serde(rename = "NetChange", deserialize_with = "de_f64")]
    pub net_change: f64,
    #[serde(rename = "PctChange", deserialize_with = "de_f64")]
    pub pct_change: f64,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct ImWmi {
    pub index: String,
    #[serde(rename = "VALUE", deserialize_with = "de_f64")]
    pub value: f64,
    #[serde(rename = "NetChange", deserialize_with = "de_f64")]
    pub net_change: f64,
    #[serde(rename = "PctChange", deserialize_with = "de_f64")]
    pub pct_change: f64,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct ImlIbor {
    #[serde(rename = "LIBOR")]
    pub libor: String,
    #[serde(rename = "USD", deserialize_with = "de_f64")]
    pub usd: f64,
    #[serde(rename = "Change1", deserialize_with = "de_f64")]
    pub change1: f64,
    #[serde(rename = "EUR", deserialize_with = "de_f64")]
    pub eur: f64,
    #[serde(rename = "Change2", deserialize_with = "de_f64")]
    pub change2: f64,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct ImEurIbor {
    #[serde(rename = "EURIBOR")]
    pub euribor: String,
    #[serde(rename = "EUR", deserialize_with = "de_f64")]
    pub eur: f64,
    #[serde(rename = "Change", deserialize_with = "de_f64")]
    pub change: f64,
    pub inserttime: String,
    pub hert: String,
}

#[derive(Debug, Deserialize)]
pub struct LmGoldRate {
    pub name: String,
    #[serde(deserialize_with = "de_f64")]
    pub weight: f64,
    #[serde(deserialize_with = "de_f64")]
    pub offer: f64,
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

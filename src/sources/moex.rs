use crate::sources::SourceSingleUrlTrait;
use rust_decimal::serde::{arbitrary_precision, arbitrary_precision_option};
use rust_decimal::Decimal;
use serde::Deserialize;

pub const API_URL: &str = "https://iss.moex.com/iss/engines/currency/markets/selt/boards/CETS/securities/AMDRUB_TOM.json?iss.meta=off&marketdata.columns=BOARDID,LAST,VALTODAY_USD&securities.columns=BOARDID,FACEVALUE&iss.clear_cache=1&iss.json=compact";

#[derive(Debug, Deserialize)]
pub struct Response {
    pub dataversion: DataVersion,
    pub marketdata: MarketData,
    pub marketdata_yields: MarketDataYields,
    pub securities: Securities,
}

#[derive(Debug, Deserialize)]
pub struct DataVersion {
    pub columns: Vec<String>,
    pub data: Vec<DataVersionData>,
}

#[derive(Debug, Deserialize)]
pub struct DataVersionData(pub i32, pub i64);

#[derive(Debug, Deserialize)]
pub struct MarketData {
    pub columns: Vec<String>,
    pub data: Vec<MarketDataData>,
}

#[derive(Debug, Deserialize)]
pub struct MarketDataData(
    pub String,
    #[serde(deserialize_with = "arbitrary_precision_option::deserialize")] pub Option<Decimal>,
    pub Option<i64>,
);

#[derive(Debug, Deserialize)]
pub struct MarketDataYields {
    pub columns: Vec<String>,
    pub data: Vec<MarketDataYieldsData>,
}

#[derive(Debug, Deserialize)]
pub struct MarketDataYieldsData(pub String, pub String);

#[derive(Debug, Deserialize)]
pub struct Securities {
    pub columns: Vec<String>,
    pub data: Vec<SecuritiesData>,
}

#[derive(Debug, Deserialize)]
pub struct SecuritiesData(
    pub String,
    #[serde(deserialize_with = "arbitrary_precision::deserialize")] pub Decimal,
);

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}

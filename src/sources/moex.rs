use crate::sources::SourceSingleUrlTrait;
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
pub struct MarketDataData(pub String, pub Option<f64>, pub Option<i64>);

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
pub struct SecuritiesData(pub String, pub f64);

impl SourceSingleUrlTrait for Response {
    fn url() -> String {
        API_URL.into()
    }
}

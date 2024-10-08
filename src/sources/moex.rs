use serde::Deserialize;
use std::env;

const ENV_TINKOFF_TOKEN: &str = "TINKOFF_TOKEN";

#[derive(Debug, Deserialize)]
pub enum Response {
    MoEx(moex::Response),
    Tinkoff(tinkoff::Response),
}

pub mod moex {
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
}

pub mod tinkoff {
    use serde::Deserialize;

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
}

impl Response {
    pub async fn get_rates(c: &reqwest::Client) -> anyhow::Result<Self> {
        let tinkoff_token = env::var(ENV_TINKOFF_TOKEN).unwrap_or_default();
        if tinkoff_token.is_empty() {
            Self::get_rates_moex(c).await
        } else {
            Self::get_rates_tinkoff(c, tinkoff_token).await
        }
    }

    async fn get_rates_moex(c: &reqwest::Client) -> anyhow::Result<Self> {
        let resp = c.get(moex::API_URL).send().await?.json().await?;
        Ok(Response::MoEx(resp))
    }

    async fn get_rates_tinkoff(c: &reqwest::Client, token: String) -> anyhow::Result<Self> {
        let req_body = tinkoff::request::Body {
            figi: tinkoff::AMDRUB_TOM_FIGI.into(),
            instrument_id: "".into(),
            depth: 1,
        };
        let resp = c
            .post(tinkoff::API_URL)
            .json(&req_body)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?
            .json()
            .await?;
        Ok(Response::Tinkoff(resp))
    }
}

use crate::source::{Currency as ModCurrency, Rate, RateType};
use anyhow::bail;
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_repr::Serialize_repr;
use std::{env, sync::LazyLock};

pub const ENV_TINKOFF_TOKEN: &str = "TINKOFF_TOKEN";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetOrderBookResponse {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quotation {
    pub units: String,
    pub nano: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub price: Quotation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct GetOrderBookRequest {
    pub depth: i32,
    pub instrument_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyResponse {
    pub instrument: Currency,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub nominal: MoneyValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyValue {
    pub units: String,
    pub nano: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct InstrumentRequest {
    pub id_type: InstrumentIdType,
    pub class_code: String,
    pub id: String,
}

#[derive(Debug, Serialize_repr)]
#[repr(u8)]
pub enum InstrumentIdType {
    InstrumentIdUnspecified = 0,
    InstrumentIdTypeFigi,
    InstrumentIdTypeTicker,
    InstrumentIdTypeUid,
    InstrumentIdTypePositionUid,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub base_url: String,
    pub path_order_book: String,
    pub path_currency: String,
    pub enabled: bool,
    pub req: GetOrderBookRequest,
}

pub async fn get_order_book(
    client: &reqwest::Client,
    config: &Config,
) -> anyhow::Result<GetOrderBookResponse> {
    let req_data = GetOrderBookRequest {
        instrument_id: config.req.instrument_id.clone(),
        depth: config.req.depth,
    };
    post(client, &config.base_url, &config.path_order_book, &req_data).await
}

pub async fn get_currency(
    client: &reqwest::Client,
    config: &Config,
) -> anyhow::Result<CurrencyResponse> {
    let req_data = InstrumentRequest {
        id_type: InstrumentIdType::InstrumentIdTypeFigi,
        class_code: "CETS".into(),
        id: config.req.instrument_id.clone(),
    };
    post(client, &config.base_url, &config.path_currency, &req_data).await
}

async fn post<T1, T2>(
    client: &reqwest::Client,
    base_url: &String,
    url_path: &String,
    req_data: &T2,
) -> anyhow::Result<T1>
where
    T1: DeserializeOwned,
    T2: Serialize + ?Sized,
{
    static TOKEN: LazyLock<String> = LazyLock::new(|| {
        let v = env::var("TINKOFF_TOKEN");
        unsafe {
            env::remove_var(ENV_TINKOFF_TOKEN);
        }
        v.unwrap_or_default()
    });
    if TOKEN.is_empty() {
        bail!(env::VarError::NotPresent);
    }
    let resp = client
        .post(format!("{}/{}", base_url, url_path))
        .json(req_data)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", TOKEN.to_string()),
        )
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(resp)
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let currency: CurrencyResponse = get_currency(client, config).await?;
    let order_book: GetOrderBookResponse = get_order_book(client, config).await?;
    let to_decimal = |units: &String, nano: i32| format!("{}.{}", units, nano).parse::<Decimal>();
    let mut rate_buy = None;
    let mut rate_sell = None;
    let nominal = to_decimal(
        &currency.instrument.nominal.units,
        currency.instrument.nominal.nano,
    )?;
    if let Some(bid) = order_book.bids.first() {
        let sell = to_decimal(&bid.price.units, bid.price.nano)?;
        if sell > Decimal::ZERO {
            rate_sell = Some(nominal / sell);
        }
    }
    if let Some(ask) = order_book.asks.first() {
        let buy = to_decimal(&ask.price.units, ask.price.nano)?;
        if buy > Decimal::ZERO {
            rate_buy = Some(nominal / buy);
        }
    }
    let mut rates = vec![];
    if rate_buy.is_some() || rate_sell.is_some() {
        rates.push(Rate {
            from: ModCurrency::rub(),
            to: ModCurrency::default(),
            rate_type: RateType::NoCash,
            buy: rate_buy,
            sell: rate_sell,
        });
    }
    Ok(rates)
}

pub use crate::source::BaseConfig as Config;
use crate::source::{de, BaseConfigTrait, Currency, Rate, RateType, USER_AGENT};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    pub result: Result,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Result {
    pub currency_rate: Vec<CurrencyRate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CurrencyRate {
    #[serde(deserialize_with = "de::empty_decimal")]
    pub buy: Option<Decimal>,
    pub cards_buy: String,
    pub cards_sell: String,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub csh_buy: Option<Decimal>,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub csh_buy_trf: Option<Decimal>,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub csh_sell: Option<Decimal>,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub csh_sell_trf: Option<Decimal>,
    #[serde(deserialize_with = "de::currency")]
    pub iso_txt: Currency,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub sell: Option<Decimal>,
}

pub(crate) async fn post<T>(client: &reqwest::Client, config: &T) -> anyhow::Result<Response>
where
    T: BaseConfigTrait,
{
    let resp = client
        .post(config.rates_url())
        .header(reqwest::header::CONTENT_LENGTH, 0)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .json()
        .await?;
    Ok(resp)
}

pub async fn collect(client: &reqwest::Client, config: &Config) -> anyhow::Result<Vec<Rate>> {
    let resp: Response = post(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for rate in resp.result.currency_rate {
        let from = rate.iso_txt;
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: rate.buy,
            sell: rate.sell,
        });
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: rate.csh_buy,
            sell: rate.csh_sell,
        });
    }
    Ok(rates)
}

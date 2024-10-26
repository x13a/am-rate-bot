use crate::source::{de, BaseConfigTrait, Currency, Rate, RateType, USER_AGENT};
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, Deserialize)]
pub struct Response {
    #[serde(rename = "getCurrencyList")]
    pub get_currency_list: GetCurrencyList,
}

#[derive(Debug, Deserialize)]
pub struct GetCurrencyList {
    #[serde(rename = "CurrencyList", default)]
    pub currency_list: Vec<CurrencyList>,
}

#[derive(Debug, Deserialize)]
pub struct CurrencyList {
    #[serde(rename = "externalId", deserialize_with = "de::currency")]
    pub external_id: Currency,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub sell: Option<Decimal>,
    #[serde(deserialize_with = "de::empty_decimal")]
    pub buy: Option<Decimal>,
    pub trf30: Trf30,
    pub trf31: Trf31,
    #[serde(rename = "CshSell", deserialize_with = "de::empty_decimal")]
    pub csh_sell: Option<Decimal>,
    #[serde(rename = "CshBuy", deserialize_with = "de::empty_decimal")]
    pub csh_buy: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct Trf30;

#[derive(Debug, Deserialize)]
pub struct Trf31;

pub mod request {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Clone, Default)]
    #[serde(rename = "request", default)]
    pub struct Request {
        #[serde(rename(serialize = "@client"))]
        pub client: String,
        #[serde(rename(serialize = "@device"))]
        pub device: String,
        #[serde(rename(serialize = "@handler"))]
        pub handler: String,
        #[serde(rename(serialize = "@lang"))]
        pub lang: String,
        #[serde(rename(serialize = "@operation"))]
        pub operation: String,
        pub accesstoken: String,
        pub id: String,
        #[serde(rename(serialize = "getCurrencyListParameters"))]
        pub get_currency_list_parameters: GetCurrencyListParameters,
        pub userid: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, Default)]
    pub struct GetCurrencyListParameters {
        pub currency: String,
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rates_url: String,
    pub enabled: bool,
    pub req: request::Request,
}

impl BaseConfigTrait for Config {
    fn rates_url(&self) -> String {
        self.rates_url.clone()
    }
}

pub trait LSoftRequest {
    fn req(&self) -> request::Request;
}

impl LSoftRequest for Config {
    fn req(&self) -> request::Request {
        self.req.clone()
    }
}

async fn post<T1, T2>(client: &reqwest::Client, config: &T2) -> anyhow::Result<T1>
where
    T1: DeserializeOwned,
    T2: BaseConfigTrait + LSoftRequest,
{
    let req = config.req();
    let req_data = request::Request {
        client: "mobile".into(),
        device: "android".into(),
        handler: "aphena".into(),
        lang: "1".into(),
        operation: "getCurrencyList".into(),
        accesstoken: "".into(),
        id: req.id.clone(),
        get_currency_list_parameters: Default::default(),
        userid: "".into(),
    };
    let body = client
        .post(config.rates_url())
        .body(quick_xml::se::to_string(&req_data)?)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let resp: T1 = quick_xml::de::from_str(&body)?;
    Ok(resp)
}

pub async fn collect<T1>(client: &reqwest::Client, config: &T1) -> anyhow::Result<Vec<Rate>>
where
    T1: BaseConfigTrait + LSoftRequest,
{
    let resp: Response = post(client, config).await?;
    let mut rates = vec![];
    let to = Currency::default();
    for item in resp.get_currency_list.currency_list {
        let from = item.external_id;
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::NoCash,
            buy: item.buy,
            sell: item.sell,
        });
        rates.push(Rate {
            from: from.clone(),
            to: to.clone(),
            rate_type: RateType::Cash,
            buy: item.csh_buy,
            sell: item.csh_sell,
        });
    }
    Ok(rates)
}

use anyhow::bail;
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize};
use std::fmt::Debug;

pub mod acba;
pub mod aeb;
pub mod alfa_by;
pub mod ameria;
pub mod amio;
pub mod ararat;
pub mod ardshin;
pub mod arm_swiss;
pub mod armsoft;
pub mod artsakh;
pub mod avosend;
pub mod byblos;
pub mod cb_am;
pub mod converse;
pub mod evoca;
pub mod fast;
pub mod hsbc;
pub mod idbank;
pub mod idpay;
pub mod ineco;
pub mod lsoft;
pub mod mellat;
pub mod mir;
pub mod moex;
pub mod sas;
pub mod unibank;
pub mod unistream;
pub mod vtb_am;

#[derive(Debug)]
pub struct Response {
    pub rates: Vec<Rate>,
}

pub trait RatesConfigTrait {
    fn rates_url(&self) -> String;
}

pub async fn get_json<T1, T2>(client: &reqwest::Client, config: &T2) -> anyhow::Result<T1>
where
    T1: DeserializeOwned,
    T2: RatesConfigTrait,
{
    let resp = client
        .get(config.rates_url())
        .send()
        .await?
        .json::<T1>()
        .await?;
    Ok(resp)
}

pub async fn get_json_for_rate_type<T1, T2>(
    client: &reqwest::Client,
    config: &T2,
    rate_type: RateType,
) -> anyhow::Result<T1>
where
    T1: DeserializeOwned,
    T2: RatesConfigTrait,
{
    match rate_type {
        RateType::NoCash | RateType::Cash => {}
        _ => bail!(Error::InvalidRateType),
    };
    let resp = client
        .get(format!("{}{}", config.rates_url(), rate_type as u8))
        .send()
        .await?
        .json::<T1>()
        .await?;
    Ok(resp)
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub acba: acba::Config,
    pub aeb: aeb::Config,
    pub ameria: ameria::Config,
    pub amio: amio::Config,
    pub ararat: ararat::Config,
    pub ardshin: ardshin::Config,
    pub arm_swiss: arm_swiss::Config,
    pub artsakh: artsakh::Config,
    pub avosend: avosend::Config,
    pub byblos: byblos::Config,
    pub cb_am: cb_am::Config,
    pub converse: converse::Config,
    pub evoca: evoca::Config,
    pub fast: fast::Config,
    pub hsbc: hsbc::Config,
    pub idbank: idbank::Config,
    pub ineco: ineco::Config,
    pub mellat: mellat::Config,
    pub mir: mir::Config,
    pub moex: moex::Config,
    pub sas: sas::Config,
    pub unibank: unibank::Config,
    pub unistream: unistream::Config,
    pub vtb_am: vtb_am::Config,
    pub idpay: idpay::Config,
    pub alfa_by: alfa_by::Config,
}

impl Config {
    pub fn is_enabled_for(&self, src: Source) -> bool {
        match src {
            Source::Acba => self.acba.enabled,
            Source::AEB => self.aeb.enabled,
            Source::Ameria => self.ameria.enabled,
            Source::Amio => self.amio.enabled,
            Source::Ararat => self.ararat.enabled,
            Source::Ardshin => self.ardshin.enabled,
            Source::ArmSwiss => self.arm_swiss.enabled,
            Source::Artsakh => self.artsakh.enabled,
            Source::Avosend => self.avosend.enabled,
            Source::Byblos => self.byblos.enabled,
            Source::CbAm => self.cb_am.enabled,
            Source::Converse => self.converse.enabled,
            Source::Evoca => self.evoca.enabled,
            Source::Fast => self.fast.enabled,
            Source::HSBC => self.hsbc.enabled,
            Source::IdBank => self.idbank.enabled,
            Source::IdPay => self.idpay.enabled,
            Source::Ineco => self.ineco.enabled,
            Source::Mellat => self.mellat.enabled,
            Source::Mir => self.mir.enabled,
            Source::MoEx => self.moex.enabled,
            Source::SAS => self.sas.enabled,
            Source::Unibank => self.unibank.enabled,
            Source::Unistream => self.unistream.enabled,
            Source::VtbAm => self.vtb_am.enabled,
            Source::AlfaBy => self.alfa_by.enabled,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RatesConfig {
    pub rates_url: String,
    pub enabled: bool,
}

impl RatesConfigTrait for RatesConfig {
    fn rates_url(&self) -> String {
        self.rates_url.clone()
    }
}

pub mod de {
    use super::{Currency, RateType};
    use rust_decimal::Decimal;
    use serde::{de, Deserialize, Deserializer};
    use std::str::FromStr;

    pub fn currency<'de, D>(deserializer: D) -> Result<Currency, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Currency::new(s))
    }

    pub fn rate_type<'de, D>(deserializer: D) -> Result<RateType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let v = RateType::from_str(&s).map_err(de::Error::custom)?;
        Ok(v)
    }

    pub fn empty_decimal<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            return Ok(None);
        }
        let f = Decimal::from_str(&s).map_err(de::Error::custom)?;
        Ok(Some(f))
    }

    pub fn decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let f = Decimal::from_str(&s).map_err(de::Error::custom)?;
        Ok(f)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    strum::EnumIter,
    strum::Display,
    Ord,
    PartialOrd,
    strum::EnumString,
    strum::EnumCount,
)]
#[strum(ascii_case_insensitive)]
pub enum Source {
    CbAm,
    MoEx,
    Acba,
    Ameria,
    Ardshin,
    ArmSwiss,
    Evoca,
    Fast,
    Ineco,
    Mellat,
    Converse,
    AEB,
    VtbAm,
    Artsakh,
    Unibank,
    Unistream,
    Amio,
    Byblos,
    IdBank,
    Ararat,
    IdPay,
    Mir,
    SAS,
    HSBC,
    Avosend,
    AlfaBy,
}

impl Source {
    pub fn prefix(&self) -> &str {
        if [Self::CbAm, Self::AlfaBy].contains(self) {
            "@"
        } else if self.is_bank() {
            "*"
        } else {
            "#"
        }
    }

    pub fn is_bank(&self) -> bool {
        ![
            Self::CbAm,
            Self::MoEx,
            Self::IdPay,
            Self::Mir,
            Self::SAS,
            Self::Avosend,
            Self::Unistream,
        ]
        .contains(self)
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash, derive_more::Display)]
pub struct Currency(pub String);

impl Currency {
    pub const AMD: &'static str = "AMD";
    pub const USD: &'static str = "USD";
    pub const EUR: &'static str = "EUR";
    pub const RUB: &'static str = "RUB";

    pub fn new<T: AsRef<str>>(s: T) -> Self {
        Self(s.as_ref().trim().to_uppercase().replace("RUR", Self::RUB))
    }

    pub fn usd() -> Self {
        Self(Self::USD.into())
    }

    pub fn eur() -> Self {
        Self(Self::EUR.into())
    }

    pub fn rub() -> Self {
        Self(Self::RUB.into())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self(Self::AMD.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, strum::EnumString)]
#[strum(ascii_case_insensitive)]
#[repr(u8)]
pub enum RateType {
    #[strum(
        serialize = "no cash",
        serialize = "non cash",
        serialize = "no_cash",
        serialize = "non_cash",
        serialize = "nocash"
    )]
    NoCash = 0,
    Cash,
    Card,
    Online,
    Cb,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid rate type")]
    InvalidRateType,
    #[error("html parse error")]
    Html,
}

#[derive(Debug, Clone)]
pub struct Rate {
    pub from: Currency,
    pub to: Currency,
    pub rate_type: RateType,
    pub buy: Option<Decimal>,
    pub sell: Option<Decimal>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use std::time::Duration;

    const TIMEOUT: u64 = 10;

    fn build_client() -> reqwest::Result<Client> {
        reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(TIMEOUT))
            .build()
    }

    fn load_config() -> anyhow::Result<Config> {
        let cfg = toml::from_str(include_str!("../../config/src.toml"))?;
        Ok(cfg)
    }

    #[tokio::test]
    async fn test_acba() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: acba::Response = get_json(&client, &config.acba).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ameria() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: ameria::Response =
            get_json_for_rate_type(&client, &config.ameria, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: ameria::Response =
            get_json_for_rate_type(&client, &config.ameria, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ardshin() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: ardshin::Response = get_json(&client, &config.ardshin).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_evoca() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: evoca::Response =
            get_json_for_rate_type(&client, &config.evoca, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: evoca::Response =
            get_json_for_rate_type(&client, &config.evoca, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_fast() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: fast::Response =
            get_json_for_rate_type(&client, &config.fast, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: fast::Response =
            get_json_for_rate_type(&client, &config.fast, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ineco() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: ineco::Response = get_json(&client, &config.ineco).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mellat() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: mellat::Response = get_json(&client, &config.mellat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_arm_swiss() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: arm_swiss::Response = get_json(&client, &config.arm_swiss).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_cb_am() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: cb_am::Response = cb_am::get(&client, &config.cb_am).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_converse() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: converse::Response = get_json(&client, &config.converse).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_aeb() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: aeb::Response = get_json(&client, &config.aeb).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_vtb_am() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: vtb_am::Response = vtb_am::get(&client, &config.vtb_am).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_artsakh() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: artsakh::Response = artsakh::get(&client, &config.artsakh).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unibank() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: unibank::Response = unibank::get(&client, &config.unibank).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_amio() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: amio::Response =
            get_json_for_rate_type(&client, &config.amio, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: amio::Response =
            get_json_for_rate_type(&client, &config.amio, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_byblos() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: byblos::Response =
            get_json_for_rate_type(&client, &config.byblos, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: byblos::Response =
            get_json_for_rate_type(&client, &config.byblos, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_idbank() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: idbank::Response = idbank::get(&client, &config.idbank).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_moex() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: moex::CurrencyResponse = moex::get_currency(&client, &config.moex).await?;
        let _: moex::GetOrderBookResponse = moex::get_order_book(&client, &config.moex).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ararat() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: ararat::Response =
            get_json_for_rate_type(&client, &config.ararat, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: ararat::Response =
            get_json_for_rate_type(&client, &config.ararat, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_idpay() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: idpay::Response = idpay::get(&client, &config.idpay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mir() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: mir::Response = get_json(&client, &config.mir).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_sas() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: sas::Response = sas::get(&client, &config.sas).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_hsbc() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: hsbc::Response = hsbc::get(&client, &config.hsbc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_avosend() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: avosend::Response = avosend::get(&client, &config.avosend).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unistream() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: unistream::Response = unistream::get(&client, &config.unistream).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_alfa_by() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: alfa_by::Response = alfa_by::get(&client, &config.alfa_by).await?;
        Ok(())
    }
}

pub use lsoft::LSoftResponse;
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fmt::Debug;

pub mod acba;
pub mod aeb;
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
pub mod kwikpay;
pub mod lsoft;
pub mod mellat;
pub mod mir;
pub mod moex;
pub mod sas;
pub mod unibank;
pub mod unistream;
pub mod vtb_am;

pub trait SourceConfigTrait {
    fn rates_url(&self) -> String;
}
pub trait JsonResponse {
    #[allow(async_fn_in_trait)]
    async fn get_rates<T1, T2>(client: &reqwest::Client, config: &T2) -> anyhow::Result<T1>
    where
        T1: DeserializeOwned,
        T2: SourceConfigTrait,
    {
        let resp = client
            .get(config.rates_url())
            .send()
            .await?
            .json::<T1>()
            .await?;
        Ok(resp)
    }
}

pub trait RateTypeJsonResponse {
    #[allow(async_fn_in_trait)]
    async fn get_rates<T1, T2>(
        client: &reqwest::Client,
        config: &T2,
        rate_type: RateType,
    ) -> anyhow::Result<T1>
    where
        T1: DeserializeOwned,
        T2: SourceConfigTrait,
    {
        match rate_type {
            RateType::NoCash | RateType::Cash => {}
            _ => Err(Error::InvalidRateType)?,
        };
        let resp = client
            .get(format!("{}{}", config.rates_url(), rate_type as u8))
            .send()
            .await?
            .json::<T1>()
            .await?;
        Ok(resp)
    }
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
    pub kwikpay: kwikpay::Config,
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
            Source::Kwikpay => self.kwikpay.enabled,
            Source::Mellat => self.mellat.enabled,
            Source::Mir => self.mir.enabled,
            Source::MoEx => self.moex.enabled,
            Source::SAS => self.sas.enabled,
            Source::UniBank => self.unibank.enabled,
            Source::UniStream => self.unistream.enabled,
            Source::VtbAm => self.vtb_am.enabled,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourceConfig {
    pub rates_url: String,
    pub enabled: bool,
}

impl SourceConfigTrait for SourceConfig {
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
    UniBank,
    UniStream,
    Amio,
    Byblos,
    IdBank,
    Ararat,
    IdPay,
    Mir,
    SAS,
    HSBC,
    Avosend,
    Kwikpay,
}

impl Source {
    pub fn prefix(&self) -> &str {
        if self.is_bank() {
            "*"
        } else {
            "#"
        }
    }

    pub fn is_bank(&self) -> bool {
        !Self::get_not_banks().contains(self)
    }

    pub fn get_not_banks() -> Vec<Self> {
        [
            Self::CbAm,
            Self::MoEx,
            Self::IdPay,
            Self::Mir,
            Self::SAS,
            Self::Avosend,
            Self::Kwikpay,
            Self::UniStream,
        ]
        .into()
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
        let _: acba::Response = acba::Response::get_rates(&client, &config.acba).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ameria() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: armsoft::Response =
            ameria::Response::get_rates(&client, &config.ameria, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response =
            ameria::Response::get_rates(&client, &config.ameria, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ardshin() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: ardshin::Response = ardshin::Response::get_rates(&client, &config.ardshin).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_evoca() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: armsoft::Response =
            evoca::Response::get_rates(&client, &config.evoca, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response =
            evoca::Response::get_rates(&client, &config.evoca, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_fast() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: fast::Response =
            fast::Response::get_rates(&client, &config.fast, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: fast::Response =
            fast::Response::get_rates(&client, &config.fast, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ineco() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: ineco::Response = ineco::Response::get_rates(&client, &config.ineco).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mellat() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: mellat::Response = mellat::Response::get_rates(&client, &config.mellat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_arm_swiss() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: arm_swiss::Response =
            arm_swiss::Response::get_rates(&client, &config.arm_swiss).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_cb_am() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: cb_am::Response = cb_am::Response::get_rates(&client, &config.cb_am).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_converse() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: converse::Response =
            converse::Response::get_rates(&client, &config.converse).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_aeb() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: aeb::Response = aeb::Response::get_rates(&client, &config.aeb).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_vtb_am() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: vtb_am::Response = vtb_am::Response::get_rates(&client, &config.vtb_am).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_artsakh() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: lsoft::Response = artsakh::Response::get_rates(&client, &config.artsakh).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unibank() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: lsoft::Response = unibank::Response::get_rates(&client, &config.unibank).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_amio() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: armsoft::Response =
            amio::Response::get_rates(&client, &config.amio, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response =
            amio::Response::get_rates(&client, &config.amio, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_byblos() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: armsoft::Response =
            byblos::Response::get_rates(&client, &config.byblos, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response =
            byblos::Response::get_rates(&client, &config.byblos, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_idbank() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: idbank::Response = idbank::Response::get_rates(&client, &config.idbank).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_moex() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: moex::Response = moex::Response::get_rates(&client, &config.moex).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ararat() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: armsoft::Response =
            ararat::Response::get_rates(&client, &config.ararat, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response =
            ararat::Response::get_rates(&client, &config.ararat, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_idpay() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: idpay::Response = idpay::Response::get_rates(&client, &config.idpay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mir() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: mir::Response = mir::Response::get_rates(&client, &config.mir).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_sas() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: sas::Response = sas::Response::get_rates(&client, &config.sas).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_hsbc() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: hsbc::Response = hsbc::Response::get_rates(&client, &config.hsbc).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_avosend() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: avosend::Response = avosend::Response::get_rates(&client, &config.avosend).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_kwikpay() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: lsoft::Response = kwikpay::Response::get_rates(&client, &config.kwikpay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unistream() -> anyhow::Result<()> {
        let client = build_client()?;
        let config = load_config()?;
        let _: lsoft::Response = unistream::Response::get_rates(&client, &config.unistream).await?;
        Ok(())
    }
}

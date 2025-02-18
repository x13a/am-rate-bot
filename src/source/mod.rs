use anyhow::ensure;
use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize};
use std::fmt::Debug;

pub mod acba;
pub mod aeb;
pub mod ameria;
pub mod amio;
pub mod ararat;
pub mod ardshin;
pub mod ardshinvest;
pub mod armsoft;
pub mod armswiss;
pub mod artsakh;
pub mod avosend;
pub mod byblos;
pub mod cb;
pub mod converse;
pub mod evoca;
pub mod fast;
pub mod idbank;
pub mod idpay;
pub mod ineco;
pub mod kwikpay;
pub mod lsoft;
pub mod mellat;
pub mod mir;
#[cfg(feature = "moex")]
pub mod moex;
pub mod sas;
pub mod unibank;
pub mod unionpay;
pub mod unistream;
pub mod vtb;

const USER_AGENT: &str = "okhttp/4.12.0";

#[derive(Debug)]
pub struct BaseResponse {
    pub rates: Vec<Rate>,
}

pub trait BaseConfigTrait {
    fn rates_url(&self) -> &str;
}

pub async fn get_json<T1, T2>(client: &reqwest::Client, config: &T2) -> anyhow::Result<T1>
where
    T1: DeserializeOwned,
    T2: BaseConfigTrait,
{
    let resp = client
        .get(config.rates_url())
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .error_for_status()?
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
    T2: BaseConfigTrait,
{
    ensure!(
        [RateType::NoCash, RateType::Cash].contains(&rate_type),
        Error::InvalidRateType
    );
    let resp = client
        .get(
            config
                .rates_url()
                .replace("%d", &(rate_type as u8).to_string()),
        )
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .error_for_status()?
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
    pub ardshinvest: ardshinvest::Config,
    pub armswiss: armswiss::Config,
    pub artsakh: artsakh::Config,
    pub avosend: avosend::Config,
    pub byblos: byblos::Config,
    pub cb: cb::Config,
    pub converse: converse::Config,
    pub evoca: evoca::Config,
    pub fast: fast::Config,
    pub idbank: idbank::Config,
    pub ineco: ineco::Config,
    pub kwikpay: kwikpay::Config,
    pub mellat: mellat::Config,
    pub mir: mir::Config,
    #[cfg(feature = "moex")]
    pub moex: moex::Config,
    pub sas: sas::Config,
    pub unibank: unibank::Config,
    pub unionpay: unionpay::Config,
    pub unistream: unistream::Config,
    pub vtb: vtb::Config,
    pub idpay: idpay::Config,
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
            Source::ArdshInvest => self.ardshinvest.enabled,
            Source::ArmSwiss => self.armswiss.enabled,
            Source::Artsakh => self.artsakh.enabled,
            Source::Avosend => self.avosend.enabled,
            Source::Byblos => self.byblos.enabled,
            Source::Cb => self.cb.enabled,
            Source::Converse => self.converse.enabled,
            Source::Evoca => self.evoca.enabled,
            Source::Fast => self.fast.enabled,
            Source::IdBank => self.idbank.enabled,
            Source::IdPay => self.idpay.enabled,
            Source::Ineco => self.ineco.enabled,
            Source::Kwikpay => self.kwikpay.enabled,
            Source::Mellat => self.mellat.enabled,
            Source::Mir => self.mir.enabled,
            #[cfg(feature = "moex")]
            Source::MOEX => self.moex.enabled,
            Source::SAS => self.sas.enabled,
            Source::Unibank => self.unibank.enabled,
            Source::UnionPay => self.unionpay.enabled,
            Source::Unistream => self.unistream.enabled,
            Source::Vtb => self.vtb.enabled,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BaseConfig {
    pub rates_url: String,
    pub enabled: bool,
}

impl BaseConfigTrait for BaseConfig {
    fn rates_url(&self) -> &str {
        &self.rates_url
    }
}

fn percent(value: Decimal, from: Decimal) -> Decimal {
    value / Decimal::ONE_HUNDRED * from
}

mod de {
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
    Cb,
    Acba,
    Ameria,
    Ardshin,
    #[strum(to_string = "ArdshInst")]
    ArdshInvest,
    ArmSwiss,
    Evoca,
    Fast,
    Ineco,
    Kwikpay,
    Mellat,
    Converse,
    AEB,
    Vtb,
    Artsakh,
    Unibank,
    UnionPay,
    Unistream,
    Amio,
    Byblos,
    IdBank,
    Ararat,
    IdPay,
    Mir,
    #[cfg(feature = "moex")]
    MOEX,
    SAS,
    Avosend,
}

impl Source {
    pub fn prefix(&self) -> &str {
        if *self == Self::Cb {
            "@"
        } else if self.is_bank() {
            "*"
        } else {
            "#"
        }
    }

    pub fn is_bank(&self) -> bool {
        ![
            Self::Cb,
            Self::IdPay,
            Self::Mir,
            #[cfg(feature = "moex")]
            Self::MOEX,
            Self::SAS,
            Self::Avosend,
            Self::Kwikpay,
            Self::UnionPay,
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
    #[error("no rates found")]
    NoRates,
}

#[derive(Debug, Clone)]
pub struct Rate {
    pub from: Currency,
    pub to: Currency,
    pub rate_type: RateType,
    pub buy: Option<Decimal>,
    pub sell: Option<Decimal>,
}

pub async fn collect(
    client: &reqwest::Client,
    config: &Config,
    src: Source,
) -> anyhow::Result<Vec<Rate>> {
    let rates = match src {
        Source::Acba => acba::collect(client, &config.acba).await?,
        Source::Ameria => ameria::collect(client, &config.ameria).await?,
        Source::Ardshin => ardshin::collect(client, &config.ardshin).await?,
        Source::ArdshInvest => ardshinvest::collect(client, &config.ardshinvest).await?,
        Source::ArmSwiss => armswiss::collect(client, &config.armswiss).await?,
        Source::Cb => cb::collect(client, &config.cb).await?,
        Source::Evoca => evoca::collect(client, &config.evoca).await?,
        Source::Fast => fast::collect(client, &config.fast).await?,
        Source::Ineco => ineco::collect(client, &config.ineco).await?,
        Source::Kwikpay => kwikpay::collect(client, &config.kwikpay).await?,
        Source::Mellat => mellat::collect(client, &config.mellat).await?,
        Source::Converse => converse::collect(client, &config.converse).await?,
        Source::AEB => aeb::collect(client, &config.aeb).await?,
        Source::Vtb => vtb::collect(client, &config.vtb).await?,
        Source::Artsakh => artsakh::collect(client, &config.artsakh).await?,
        Source::Unibank => unibank::collect(client, &config.unibank).await?,
        Source::Amio => amio::collect(client, &config.amio).await?,
        Source::Byblos => byblos::collect(client, &config.byblos).await?,
        Source::IdBank => idbank::collect(client, &config.idbank).await?,
        Source::Ararat => ararat::collect(client, &config.ararat).await?,
        Source::IdPay => idpay::collect(client, &config.idpay).await?,
        Source::Mir => mir::collect(client, &config.mir).await?,
        #[cfg(feature = "moex")]
        Source::MOEX => moex::collect(client, &config.moex).await?,
        Source::SAS => sas::collect(client, &config.sas).await?,
        Source::Avosend => avosend::collect(client, &config.avosend).await?,
        Source::UnionPay => unionpay::collect(client, &config.unionpay).await?,
        Source::Unistream => unistream::collect(client, &config.unistream).await?,
    };
    Ok(rates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::{sync::LazyLock, time::Duration};

    static CFG: LazyLock<Config> =
        LazyLock::new(|| toml::from_str(include_str!("../../config/config.toml")).unwrap());

    fn build_client(cfg: &Config) -> reqwest::Result<reqwest::Client> {
        reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(cfg.bot.reqwest_timeout))
            .build()
    }

    #[tokio::test]
    async fn test_acba() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Acba).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ameria() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Ameria).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ardshin() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Ardshin).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ardshinvest() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::ArdshInvest).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_evoca() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Evoca).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_fast() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Fast).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ineco() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Ineco).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_kwikpay() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Kwikpay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mellat() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Mellat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_armswiss() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::ArmSwiss).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_cb() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Cb).await?;
        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(feature = "github_ci", ignore)]
    async fn test_converse() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Converse).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_aeb() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::AEB).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_vtb() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Vtb).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_artsakh() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Artsakh).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unibank() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Unibank).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_amio() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Amio).await?;
        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(feature = "github_ci", ignore)]
    async fn test_byblos() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Byblos).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_idbank() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::IdBank).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ararat() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Ararat).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_idpay() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::IdPay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mir() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Mir).await?;
        Ok(())
    }

    #[cfg(feature = "moex")]
    #[tokio::test]
    async fn test_moex() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::MOEX).await?;
        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(feature = "github_ci", ignore)]
    async fn test_sas() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::SAS).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_avosend() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Avosend).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unionpay() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::UnionPay).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unistream() -> anyhow::Result<()> {
        let client = build_client(&CFG)?;
        let _ = collect(&client, &CFG.src, Source::Unistream).await?;
        Ok(())
    }
}

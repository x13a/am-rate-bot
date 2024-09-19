use crate::sources::utils::de_currency;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub mod acba;
pub mod ameria;
pub mod ardshin;
pub mod arm_swiss;
pub mod cba;
pub mod evoca;
pub mod fast;
pub mod ineco;
pub mod mellat;
mod utils;

pub trait SourceSingleUrlTrait {
    fn url() -> String;
    async fn get_rates<T>(c: &reqwest::Client) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let resp = c.get(Self::url()).send().await?.json::<T>().await?;
        Ok(resp)
    }
}

pub trait SourceCashUrlTrait {
    fn url_cash() -> String;
    fn url_no_cash() -> String;
    async fn get_rates<T>(c: &reqwest::Client, rate_type: RateType) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let url = match rate_type {
            RateType::NoCash => Self::url_no_cash(),
            RateType::Cash => Self::url_cash(),
            _ => return Err(Error::InvalidRateType),
        };
        let resp = c.get(url).send().await?.json::<T>().await?;
        Ok(resp)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    CBA,
    Acba,
    Ameria,
    Ardshin,
    ArmSwiss,
    Evoca,
    Fast,
    Ineco,
    Mellat,
}

impl Source {
    pub fn iter() -> impl Iterator<Item = Source> {
        [
            Self::CBA,
            Self::Acba,
            Self::Ameria,
            Self::Ardshin,
            Self::ArmSwiss,
            Self::Evoca,
            Self::Fast,
            Self::Ineco,
            Self::Mellat,
        ]
        .iter()
        .copied()
    }

    pub fn prefix(&self) -> &str {
        match self {
            Self::CBA => "#",
            _ => "*",
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Source::CBA => "CBA".into(),
            Source::Acba => "Acba".into(),
            Source::Ameria => "Ameria".into(),
            Source::Ardshin => "Ardshin".into(),
            Source::ArmSwiss => "ArmSwiss".into(),
            Source::Evoca => "Evoca".into(),
            Source::Fast => "Fast".into(),
            Source::Ineco => "Ineco".into(),
            Source::Mellat => "Mellat".into(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Currency(String);

impl Currency {
    pub const AMD: &'static str = "AMD";
    pub const USD: &'static str = "USD";
    pub const EUR: &'static str = "EUR";
    pub const RUB: &'static str = "RUB";
    pub fn new(s: &str) -> Self {
        Self(s.trim().to_uppercase().replace("RUR", Self::RUB))
    }

    pub fn cross_to_currencies(&self) -> Option<(Self, Self)> {
        self.0
            .split_once('/')
            .map(|(a, b)| (Self::new(a), Self::new(b)))
    }

    pub fn base() -> Self {
        Self(Self::AMD.into())
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
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Currency {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Currency::new(s))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RateType {
    NoCash = 0,
    Cash = 1,
    Card = 2,
    Online = 3,
    Cross = 4,
    CB = 5,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExchangeRate {
    #[serde(rename = "CBRate")]
    pub cb_rate: f64,
    #[serde(deserialize_with = "de_currency")]
    pub currency: Currency,
    pub purchase: f64,
    pub rate_for: u16,
    pub sale: f64,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    De(quick_xml::DeError),
    InvalidRateType,
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<quick_xml::DeError> for Error {
    fn from(e: quick_xml::DeError) -> Self {
        Self::De(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub(crate) mod tests {
    use super::*;
    use reqwest::Client;
    use std::time::Duration;

    const TIMEOUT: u64 = 10;

    pub(crate) fn build_client() -> reqwest::Result<Client> {
        reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(TIMEOUT))
            .build()
    }
    #[tokio::test]
    async fn test_acba() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: acba::Response = acba::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ameria() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: ameria::Response = ameria::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: ameria::Response = ameria::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ardshin() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: ardshin::Response = ardshin::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_evoca() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: evoca::Response = evoca::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: evoca::Response = evoca::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_fast() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: fast::Response = fast::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: fast::Response = fast::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ineco() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: ineco::Response = ineco::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mellat() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: mellat::Response = mellat::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_arm_swiss() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: arm_swiss::Response = arm_swiss::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_cba() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: cba::Response = cba::Response::get_rates(&c).await?;
        Ok(())
    }
}

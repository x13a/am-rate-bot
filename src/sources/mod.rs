pub use lsoft::SourceAphenaTrait;
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use strum::EnumIter;

pub mod acba;
pub mod aeb;
pub mod ameria;
pub mod amio;
pub mod ararat;
pub mod ardshin;
pub mod arm_swiss;
pub mod armsoft;
pub mod artsakh;
pub mod byblos;
pub mod cb_am;
pub mod converse;
pub mod evoca;
pub mod fast;
pub mod hsbc;
pub mod idbank;
pub mod ineco;
pub mod lsoft;
pub mod mellat;
pub mod mir;
pub mod moex;
pub mod sas;
pub mod unibank;
mod utils;
pub mod vtb_am;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, strum::Display)]
pub enum Source {
    #[strum(serialize = "CBAM")]
    CbAm,
    #[strum(serialize = "MOEX'")]
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
    #[strum(serialize = "VTB AM")]
    VtbAm,
    Artsakh,
    UniBank,
    #[strum(serialize = "AMIO")]
    Amio,
    Byblos,
    IdBank,
    Ararat,
    #[strum(serialize = "IdPay'")]
    IdPay,
    #[strum(serialize = "MIR")]
    Mir,
    #[strum(serialize = "SAS")]
    Sas,
    #[strum(serialize = "HSBC")]
    Hsbc,
}

impl Source {
    pub fn prefix(&self) -> &str {
        if Self::get_not_banks().contains(self) {
            "#"
        } else {
            "*"
        }
    }

    pub fn get_not_banks() -> Vec<Self> {
        [Self::CbAm, Self::MoEx, Self::IdPay, Self::Mir, Self::Sas].into()
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Currency(String);

impl Currency {
    pub const AMD: &'static str = "AMD";
    pub const USD: &'static str = "USD";
    pub const EUR: &'static str = "EUR";
    pub const RUB: &'static str = "RUB";

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

impl<S> From<S> for Currency
where
    S: AsRef<str>,
{
    fn from(s: S) -> Self {
        Self(s.as_ref().trim().to_uppercase().replace("RUR", Self::RUB))
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self(Self::AMD.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RateType {
    NoCash = 0,
    Cash = 1,
    Card = 2,
    Online = 3,
    CB = 4,
}

impl FromStr for RateType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rt = match s.to_uppercase().as_str() {
            "NO CASH" | "NON CASH" | "NO_CASH" | "NON_CASH" => Self::NoCash,
            "CASH" => Self::Cash,
            "CARD" => Self::Card,
            "ONLINE" => Self::Online,
            "CB" => Self::CB,
            _ => return Err(Error::InvalidRateType),
        };
        Ok(rt)
    }
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Xml(quick_xml::DeError),
    InvalidRateType,
    Html,
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<quick_xml::DeError> for Error {
    fn from(e: quick_xml::DeError) -> Self {
        Self::Xml(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub struct Rate {
    pub from: Currency,
    pub to: Currency,
    pub rate_type: RateType,
    pub buy: Option<f64>,
    pub sell: Option<f64>,
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
    #[tokio::test]
    async fn test_acba() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: acba::Response = acba::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ameria() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: armsoft::Response = ameria::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = ameria::Response::get_rates(&c, RateType::Cash).await?;
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
        let _: armsoft::Response = evoca::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = evoca::Response::get_rates(&c, RateType::Cash).await?;
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
    async fn test_cb_am() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: cb_am::Response = cb_am::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_converse() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: converse::Response = converse::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_aeb() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: aeb::Response = aeb::Response::get_rates(&c).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_vtb_am() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: vtb_am::Response = vtb_am::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_artsakh() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: lsoft::Response = artsakh::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unibank() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: lsoft::Response = unibank::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_amio() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: armsoft::Response = amio::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = amio::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_byblos() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: armsoft::Response = byblos::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = byblos::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_idbank() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: idbank::Response = idbank::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_moex() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: moex::Response = moex::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ararat() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: armsoft::Response = ararat::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = ararat::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_idpay() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: idbank::Response = idbank::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mir() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: mir::Response = mir::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_sas() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: sas::Response = sas::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_hsbc() -> Result<(), Box<dyn std::error::Error>> {
        let c = build_client()?;
        let _: hsbc::Response = hsbc::Response::get_rates(&c).await?;
        Ok(())
    }
}

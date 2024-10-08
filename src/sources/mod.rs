pub use lsoft::SourceAphenaTrait;
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
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

    #[allow(async_fn_in_trait)]
    async fn get_rates<T>(c: &reqwest::Client) -> anyhow::Result<T>
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

    #[allow(async_fn_in_trait)]
    async fn get_rates<T>(c: &reqwest::Client, rate_type: RateType) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let url = match rate_type {
            RateType::NoCash => Self::url_no_cash(),
            RateType::Cash => Self::url_cash(),
            _ => Err(Error::InvalidRateType)?,
        };
        let resp = c.get(url).send().await?.json::<T>().await?;
        Ok(resp)
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
    #[strum(to_string = "CBAM")]
    CbAm,
    #[strum(to_string = "MOEX'", serialize = "moex")]
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
    #[strum(to_string = "VTB AM", serialize = "vtbam")]
    VtbAm,
    Artsakh,
    UniBank,
    #[strum(to_string = "AMIO")]
    Amio,
    Byblos,
    IdBank,
    Ararat,
    #[strum(to_string = "IdPay'", serialize = "idpay")]
    IdPay,
    #[strum(to_string = "MIR")]
    Mir,
    #[strum(to_string = "SAS")]
    Sas,
    #[strum(to_string = "HSBC")]
    Hsbc,
    Avosend,
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
        [
            Self::CbAm,
            Self::MoEx,
            Self::IdPay,
            Self::Mir,
            Self::Sas,
            Self::Avosend,
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
    #[tokio::test]
    async fn test_acba() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: acba::Response = acba::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ameria() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: armsoft::Response = ameria::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = ameria::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ardshin() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: ardshin::Response = ardshin::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_evoca() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: armsoft::Response = evoca::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = evoca::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_fast() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: fast::Response = fast::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: fast::Response = fast::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ineco() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: ineco::Response = ineco::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mellat() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: mellat::Response = mellat::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_arm_swiss() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: arm_swiss::Response = arm_swiss::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_cb_am() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: cb_am::Response = cb_am::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_converse() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: converse::Response = converse::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_aeb() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: aeb::Response = aeb::Response::get_rates(&c).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_vtb_am() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: vtb_am::Response = vtb_am::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_artsakh() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: lsoft::Response = artsakh::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_unibank() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: lsoft::Response = unibank::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_amio() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: armsoft::Response = amio::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = amio::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_byblos() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: armsoft::Response = byblos::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = byblos::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_idbank() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: idbank::Response = idbank::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_moex() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: moex::Response = moex::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ararat() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: armsoft::Response = ararat::Response::get_rates(&c, RateType::NoCash).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;
        let _: armsoft::Response = ararat::Response::get_rates(&c, RateType::Cash).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_idpay() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: idpay::Response = idpay::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_mir() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: mir::Response = mir::Response::get_rates(&c).await?;
        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn test_sas() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: sas::Response = sas::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_hsbc() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: hsbc::Response = hsbc::Response::get_rates(&c).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_avosend() -> anyhow::Result<()> {
        let c = build_client()?;
        let _: avosend::Response = avosend::Response::get_rates(&c).await?;
        Ok(())
    }
}

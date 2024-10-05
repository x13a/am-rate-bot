use crate::sources::{Currency, RateType};
use rust_decimal::Decimal;
use serde::{de, Deserialize, Deserializer};
use std::str::FromStr;

pub(crate) fn de_currency<'de, D>(deserializer: D) -> Result<Currency, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Currency::new(s))
}

pub(crate) fn de_rate_type<'de, D>(deserializer: D) -> Result<RateType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let v = RateType::from_str(&s).map_err(de::Error::custom)?;
    Ok(v)
}

pub(crate) fn de_empty_decimal<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
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

pub(crate) fn de_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let f = Decimal::from_str(&s).map_err(de::Error::custom)?;
    Ok(f)
}

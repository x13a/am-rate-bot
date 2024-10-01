use crate::sources::{Currency, RateType};
use serde::{de, Deserialize, Deserializer};
use std::str::FromStr;

pub(crate) fn de_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s = String::deserialize(deserializer)?;
    for c in [",", "%"] {
        s = s.replace(c, "");
    }
    let f = s.parse::<f64>().map_err(de::Error::custom)?;
    Ok(f)
}

pub(crate) fn de_currency<'de, D>(deserializer: D) -> Result<Currency, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Currency::from(&s))
}

pub(crate) fn de_rate_type<'de, D>(deserializer: D) -> Result<RateType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let v = RateType::from_str(&s).map_err(de::Error::custom)?;
    Ok(v)
}

pub(crate) fn de_option_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    let f = s.parse::<f64>().map_err(de::Error::custom)?;
    Ok(Some(f))
}

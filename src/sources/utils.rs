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
    Ok(Currency::from_str(&s).expect("currency not supported"))
}

pub(crate) fn de_rate_type<'de, D>(deserializer: D) -> Result<RateType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let rt = RateType::from_str(&s).map_err(de::Error::custom)?;
    Ok(rt)
}

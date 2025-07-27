use anyhow::Result;
use serde::{Deserialize, Serialize};

pub fn from_str<T>(line: &str) -> Result<T>
where
    for<'a> T: Deserialize<'a>,
{
    serde_json::from_str(line).map_err(anyhow::Error::new)
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    serde_json::to_string(value).map_err(anyhow::Error::new)
}

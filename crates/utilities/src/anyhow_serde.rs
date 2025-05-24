use anyhow::Result;
use serde::{Deserialize, Serialize};

pub fn from_str<T: for<'a> Deserialize<'a>>(line: &String) -> Result<T> {
    serde_json::from_str(line).map_err(anyhow::Error::new)
}

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(anyhow::Error::new)
}

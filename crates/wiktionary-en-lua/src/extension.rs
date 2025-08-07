use mlua::{FromLua, Lua, Value};
use std::any::type_name;

use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum ExtensionErrorType {
    UnknownOption,
    UnknownError,
}

impl ExtensionErrorType {
    pub fn from(code: &str) -> Self {
        match code {
            "unknown_option" => Self::UnknownOption,
            _ => Self::UnknownError,
        }
    }
}

impl Display for ExtensionErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::UnknownOption => write!(f, "unknown extension option"),
            Self::UnknownError => write!(f, "unknown extension error"),
        }
    }
}

#[derive(Debug)]
pub struct ExtensionResult {
    pub result: String,
    pub error: Option<ExtensionErrorType>,
}

impl FromLua for ExtensionResult {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        if let Some(table) = value.as_table() {
            return Ok(Self {
                result: table.get("result")?,
                error: table.get("error")?,
            });
        }
        Err(mlua::Error::FromLuaConversionError {
            from: "table",
            to: type_name::<Self>().to_string(),
            message: None,
        })
    }
}

impl FromLua for ExtensionErrorType {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        if let Some(error_type) = value.as_string() {
            return Ok(ExtensionErrorType::from(&error_type.to_str()?));
        }
        Err(mlua::Error::FromLuaConversionError {
            from: "error_type",
            to: type_name::<Self>().to_string(),
            message: None,
        })
    }
}

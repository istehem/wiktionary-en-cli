use crate::extension::{ExtensionErrorType, ExtensionResult};
use mlua::{FromLua, Lua, Value};
use std::any::type_name;

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

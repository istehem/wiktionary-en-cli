use mlua::Lua;
use mlua::Value;
use mlua::{FromLua, IntoLua};
use std::any::type_name;

use crate::result::{DictionaryResult, ExtensionErrorType, ExtensionResult};

impl IntoLua for DictionaryResult {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let dictionary_result = lua.create_table()?;
        dictionary_result.set("word", self.word)?;
        dictionary_result.set("did_you_mean", self.did_you_mean)?;
        dictionary_result.set("hits", self.hits)?;
        Ok(mlua::Value::Table(dictionary_result))
    }
}

impl FromLua for DictionaryResult {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        if let Some(table) = value.as_table() {
            return Ok(Self {
                word: table.get("word")?,
                did_you_mean: table.get("did_you_mean")?,
                hits: table.get("hits")?,
            });
        }
        Err(mlua::Error::RuntimeError(
            "no result could be interpreted".to_string(),
        ))
    }
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
        if let Some(integer) = value.as_integer() {
            return Ok(ExtensionErrorType::from(integer));
        }
        Err(mlua::Error::FromLuaConversionError {
            from: "integer",
            to: type_name::<Self>().to_string(),
            message: None,
        })
    }
}

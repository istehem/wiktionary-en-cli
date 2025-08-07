use mlua::Lua;
use mlua::Value;
use mlua::{FromLua, IntoLua};

use crate::result::{DictionaryResult, DidYouMean};

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

impl IntoLua for DidYouMean {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let did_you_mean = lua.create_table()?;
        did_you_mean.set("searched_for", self.searched_for)?;
        did_you_mean.set("suggestion", self.suggestion)?;
        Ok(mlua::Value::Table(did_you_mean))
    }
}

impl FromLua for DidYouMean {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        if let Some(did_you_mean) = value.as_table() {
            let entry = DidYouMean {
                searched_for: did_you_mean.get("searched_for")?,
                suggestion: did_you_mean.get("suggestion")?,
            };
            return Ok(entry);
        }
        Err(mlua::Error::RuntimeError(
            "no valid did-you-mean definition found in lua".to_string(),
        ))
    }
}

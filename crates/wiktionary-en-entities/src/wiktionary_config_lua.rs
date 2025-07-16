use mlua::FromLua;
use mlua::Lua;
use mlua::Value;

use crate::wiktionary_config::Config;

impl FromLua for Config {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let table = value.as_table();
        match table {
            Some(table) => {
                let language_code: String = table.get("language")?;
                Ok(Config {
                    language: language_code
                        .parse()
                        .map_err(|err: anyhow::Error| mlua::Error::RuntimeError(err.to_string()))?,
                })
            }
            None => Ok(Config::default()),
        }
    }
}

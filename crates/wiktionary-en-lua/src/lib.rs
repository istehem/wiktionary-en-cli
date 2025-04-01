use anyhow::{bail, Result};
use mlua::{FromLua, IntoLua, Lua, Value};
use utilities::language::*;
use utilities::DICTIONARY_CONFIG;

#[derive(Default, Clone)]
pub struct Config {
    pub message: String,
    pub language: Language,
}

impl Config {
    fn new() -> Self {
        Self {
            message: String::from("default"),
            language: Language::default(),
        }
    }
}

impl FromLua for Config {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let table = value.as_table();
        return match table {
            Some(table) => {
                let message: String = table.get("message")?;
                let language_code: String = table.get("language")?;
                return Ok(Config {
                    message: message,
                    language: Language::from_str_or_default(&language_code),
                });
            }
            None => Ok(Config::new()),
        };
    }
}

impl IntoLua for Config {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let config = lua.create_table()?;
        config.set("message", self.message)?;
        return Ok(mlua::Value::Table(config));
    }
}

fn load_config(lua: &Lua) -> mlua::Result<Config> {
    lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?)
        .exec()?;
    let config: mlua::Value = lua.globals().get("config")?;
    if let Some(config) = config.as_function() {
        return config.call(());
    }
    return Config::from_lua(config, lua);
}

pub fn do_load_config() -> Result<Config> {
    let lua = Lua::new();
    match load_config(&lua) {
        Ok(result) => return Ok(result),
        Err(err) => bail!(err.to_string()),
    }
}

fn one_plus_one(lua: &Lua) -> mlua::Result<u8> {
    lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?)
        .exec()?;
    let one_plus_one: mlua::Function = lua.globals().get("one_plus_one")?;
    let result: u8 = one_plus_one.call(())?;
    println!("lua function returned: {}", result);
    return Ok(result);
}

pub fn do_one_plus_one() -> Result<u8> {
    let lua = Lua::new();
    match one_plus_one(&lua) {
        Ok(result) => return Ok(result),
        Err(err) => bail!(err.to_string()),
    }
}

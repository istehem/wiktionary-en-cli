use anyhow::{anyhow, Result};
use mlua::{FromLua, Lua};
use utilities::DICTIONARY_CONFIG;
use wiktionary_en_entities::config::Config;

const LUA_CONFIGURATION_ERROR: &str = "Lua Configuration Error";

pub struct ConfigHandler {
    pub config: Config,
}

impl ConfigHandler {
    pub fn init() -> Result<Self> {
        let lua = Lua::new();
        match init_lua_config(&lua) {
            Ok(_) => {
                let config = match get_config(&lua) {
                    Ok(result) => Ok(result),
                    Err(err) => Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR)),
                }?;
                Ok(Self { config })
            }
            Err(err) => Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR)),
        }
    }
}

fn init_lua_config(lua: &Lua) -> mlua::Result<()> {
    lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?)
        .exec()
}

fn get_config(lua: &Lua) -> mlua::Result<Config> {
    let config: mlua::Value = get_config_as_lua_value(lua)?;
    if let Some(config) = config.as_function() {
        return config.call(());
    }
    Config::from_lua(config, lua)
}

fn get_config_as_lua_value(lua: &Lua) -> mlua::Result<mlua::Value> {
    lua.globals().get("config")
}

use anyhow::{anyhow, Result};
use colored::Colorize;
use mlua::{FromLua, Function, Lua, Value};
use utilities::colored_string_utils;
use utilities::language::*;
use utilities::DICTIONARY_CONFIG;
use utilities::LUA_DIR;
use wiktionary_en_entities::wiktionary_entity::*;

const LUA_CONFIGURATION_ERROR: &str = "Lua Configuration Error";

#[derive(Default, Clone)]
pub struct Config {
    pub language: Option<Language>,
}

pub struct ConfigHandler {
    lua: Lua,
    pub config: Config,
}

impl Config {
    fn new() -> Self {
        Self { language: None }
    }
}

impl ConfigHandler {
    pub fn init() -> Result<Self> {
        let mut config_handler = Self::init_lua()?;
        config_handler.config = config_handler.load_config()?;
        Ok(config_handler)
    }

    fn init_lua() -> Result<Self> {
        let lua = Lua::new();

        match init_lua(&lua) {
            Ok(_) => Ok(Self {
                lua,
                config: Config::default(),
            }),
            Err(err) => Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR)),
        }
    }

    fn intercept(&self, dictionary_entry: &DictionaryEntry) -> Result<Option<DictionaryEntry>> {
        match intercept(&self.lua, dictionary_entry) {
            Ok(entry) => Ok(entry),
            Err(err) => Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR)),
        }
    }

    pub fn intercept_wiktionary_result(
        &self,
        result: &Vec<DictionaryEntry>,
    ) -> Result<Option<Vec<DictionaryEntry>>> {
        let mut intercepted_result = Vec::new();
        for entry in result {
            if let Some(entry) = self.intercept(entry)? {
                intercepted_result.push(entry);
            } else {
                return Ok(None);
            }
        }
        Ok(Some(intercepted_result))
    }

    fn format(&self, dictionary_entry: &DictionaryEntry) -> Result<Option<String>> {
        match format(&self.lua, dictionary_entry) {
            Ok(entry) => Ok(entry),
            Err(err) => Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR)),
        }
    }

    pub fn format_wiktionary_result(
        &self,
        result: &Vec<DictionaryEntry>,
    ) -> Result<Option<Vec<String>>> {
        let mut formatted_entries = Vec::new();
        for entry in result {
            if let Some(formatted_entry) = self.format(entry)? {
                formatted_entries.push(formatted_entry);
            } else {
                return Ok(None);
            }
        }
        Ok(Some(formatted_entries))
    }

    fn load_config(&self) -> Result<Config> {
        match load_config(&self.lua) {
            Ok(result) => Ok(result),
            Err(err) => Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR)),
        }
    }
}

fn init_lua(lua: &Lua) -> mlua::Result<()> {
    load_lua_library(lua)?;
    lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?)
        .exec()?;
    load_lua_api(lua)
}

fn intercept(
    lua: &Lua,
    dictionary_entry: &DictionaryEntry,
) -> mlua::Result<Option<DictionaryEntry>> {
    let intercept: mlua::Value = lua.globals().get("intercept")?;
    if let Some(intercept) = intercept.as_function() {
        return Ok(Some(intercept.call(dictionary_entry.clone())?));
    }

    Ok(None)
}

fn format(lua: &Lua, dictionary_entry: &DictionaryEntry) -> mlua::Result<Option<String>> {
    let format_fn: mlua::Value = lua.globals().get("format")?;
    if let Some(format_fn) = format_fn.as_function() {
        return Ok(Some(format_fn.call(dictionary_entry.clone())?));
    }

    Ok(None)
}

impl FromLua for Config {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let table = value.as_table();
        match table {
            Some(table) => {
                let language_code: String = table.get("language")?;
                Ok(Config {
                    language: language_code.parse().ok(),
                })
            }
            None => Ok(Config::new()),
        }
    }
}

fn load_config(lua: &Lua) -> mlua::Result<Config> {
    let config: mlua::Value = lua.globals().get("config")?;
    if let Some(config) = config.as_function() {
        return config.call(());
    }
    Config::from_lua(config, lua)
}

fn load_lua_library(lua: &Lua) -> mlua::Result<()> {
    let package: mlua::Table = lua.globals().get("package")?;
    let path: String = package.get("path")?;
    package.set("path", format!("{};{}/?.lua", path, LUA_DIR!()))?;
    Ok(())
}

fn load_lua_api(lua: &Lua) -> mlua::Result<()> {
    let wiktionary_api = lua.create_table()?;
    let apply_color_fn = apply_color(lua)?;
    wiktionary_api.set("apply_color", apply_color_fn)?;
    let apply_style_fn = apply_style(lua)?;
    wiktionary_api.set("apply_style", apply_style_fn)?;
    let horizontal_line_fn = horizontal_line(lua)?;
    wiktionary_api.set("horizontal_line", horizontal_line_fn)?;
    let to_pretty_string_fn = to_pretty_string(lua)?;
    wiktionary_api.set("to_pretty_string", to_pretty_string_fn)?;
    let wrap_text_at_fn = wrap_text_at(lua)?;
    wiktionary_api.set("wrap_text_at", wrap_text_at_fn)?;
    let indent_fn = indent(lua)?;
    wiktionary_api.set("indent", indent_fn)?;

    lua.globals().set("api", wiktionary_api)?;
    Ok(())
}

fn apply_color(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (text, color): (String, String)| {
        let colored_text = match color.to_lowercase().as_str() {
            "red" => text.red(),
            "green" => text.green(),
            "blue" => text.blue(),
            "yellow" => text.yellow(),
            "cyan" => text.cyan(),
            "magenta" => text.magenta(),
            "white" => text.white(),
            "black" => text.black(),
            _ => text.into(), // Default to the original text if color is unknown
        };
        Ok(colored_text.to_string())
    })
}

fn apply_style(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (text, style): (String, String)| {
        let style_text = match style.to_lowercase().as_str() {
            "bold" => text.bold(),
            "dimmed" => text.dimmed(),
            "underline" => text.underline(),
            "reversed" => text.reversed(),
            "italic" => text.italic(),
            "blink" => text.blink(),
            "hidden" => text.hidden(),
            "strikethrough" => text.strikethrough(),
            _ => text.into(), // Default to the original text if color is unknown
        };
        Ok(style_text.to_string())
    })
}

fn horizontal_line(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, ()| Ok(colored_string_utils::horizontal_line().to_string()))
}

fn wrap_text_at(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (text, width): (String, usize)| {
        Ok(colored_string_utils::wrap(&text.into(), width).to_string())
    })
}

fn indent(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(
        |_, text: String| Ok(colored_string_utils::indent(&text.into()).to_string()),
    )
}

fn to_pretty_string(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, dictionary_entry: DictionaryEntry| {
        Ok(dictionary_entry.to_pretty_string())
    })
}

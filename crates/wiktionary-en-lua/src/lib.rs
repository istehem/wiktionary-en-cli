use anyhow::{anyhow, Result};
use colored::Colorize;
use mlua::{FromLua, Function, Lua, Value};
use utilities::colored_string_utils;
use utilities::language::*;
use utilities::DICTIONARY_CONFIG;
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
        return Ok(config_handler);
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
            Err(err) => {
                return Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR));
            }
        }
    }

    pub fn intercept_wiktionary_result(
        &self,
        result: &Vec<DictionaryEntry>,
    ) -> Result<Option<Vec<DictionaryEntry>>> {
        let mut intercepted_result = Vec::new();
        for entry in result {
            if let Some(entry) = self.intercept(&entry)? {
                intercepted_result.push(entry);
            } else {
                return Ok(None);
            }
        }
        return Ok(Some(intercepted_result));
    }

    fn format(&self, dictionary_entry: &DictionaryEntry) -> Result<Option<String>> {
        match format(&self.lua, dictionary_entry) {
            Ok(entry) => Ok(entry),
            Err(err) => {
                return Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR));
            }
        }
    }

    pub fn format_wiktionary_result(
        &self,
        result: &Vec<DictionaryEntry>,
    ) -> Result<Option<Vec<String>>> {
        let mut formatted_entries = Vec::new();
        for entry in result {
            if let Some(formatted_entry) = self.format(&entry)? {
                formatted_entries.push(formatted_entry);
            } else {
                return Ok(None);
            }
        }
        return Ok(Some(formatted_entries));
    }

    fn load_config(&self) -> Result<Config> {
        match load_config(&self.lua) {
            Ok(result) => {
                return Ok(result);
            }
            Err(err) => Err(anyhow!("{}", err).context(LUA_CONFIGURATION_ERROR)),
        }
    }
}

fn init_lua(lua: &Lua) -> mlua::Result<()> {
    lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?)
        .exec()?;
    return load_lua_api(&lua);
}

fn intercept(
    lua: &Lua,
    dictionary_entry: &DictionaryEntry,
) -> mlua::Result<Option<DictionaryEntry>> {
    let intercept: mlua::Value = lua.globals().get("intercept")?;
    if let Some(intercept) = intercept.as_function() {
        return Ok(Some(intercept.call(dictionary_entry.clone())?));
    }

    return Ok(None);
}

fn format(lua: &Lua, dictionary_entry: &DictionaryEntry) -> mlua::Result<Option<String>> {
    let format_fn: mlua::Value = lua.globals().get("format")?;
    if let Some(format_fn) = format_fn.as_function() {
        return Ok(Some(format_fn.call(dictionary_entry.clone())?));
    }

    return Ok(None);
}

impl FromLua for Config {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let table = value.as_table();
        return match table {
            Some(table) => {
                let language_code: String = table.get("language")?;
                return Ok(Config {
                    language: language_code.parse().ok(),
                });
            }
            None => Ok(Config::new()),
        };
    }
}

fn load_config(lua: &Lua) -> mlua::Result<Config> {
    let config: mlua::Value = lua.globals().get("config")?;
    if let Some(config) = config.as_function() {
        return config.call(());
    }
    return Config::from_lua(config, lua);
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
    return Ok(());
}

fn apply_color(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (text, color): (String, String)| {
        let colored_text = match color.to_lowercase().as_str() {
            "red" => text.red().to_string(),
            "green" => text.green().to_string(),
            "blue" => text.blue().to_string(),
            "yellow" => text.yellow().to_string(),
            "cyan" => text.cyan().to_string(),
            "magenta" => text.magenta().to_string(),
            "white" => text.white().to_string(),
            "black" => text.black().to_string(),
            _ => text.to_string(), // Default to the original text if color is unknown
        };
        Ok(colored_text)
    })
}

fn apply_style(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (text, style): (String, String)| {
        let style_text = match style.to_lowercase().as_str() {
            "bold" => text.bold().to_string(),
            "dimmed" => text.dimmed().to_string(),
            "underline" => text.underline().to_string(),
            "reversed" => text.reversed().to_string(),
            "italic" => text.italic().to_string(),
            "blink" => text.blink().to_string(),
            "hidden" => text.hidden().to_string(),
            "strikethrough" => text.strikethrough().to_string(),
            _ => text.to_string(), // Default to the original text if color is unknown
        };
        Ok(style_text)
    })
}

fn horizontal_line(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, ()| {
        return Ok(colored_string_utils::horizontal_line().to_string());
    })
}

fn wrap_text_at(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, (text, width): (String, usize)| {
        return Ok(colored_string_utils::wrap(&text.into(), width).to_string());
    })
}

fn indent(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, text: String| {
        return Ok(colored_string_utils::indent(&text.into()).to_string());
    })
}

fn to_pretty_string(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, dictionary_entry: DictionaryEntry| {
        return Ok(dictionary_entry.to_pretty_string());
    })
}

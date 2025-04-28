use anyhow::{bail, Result};
use colored::Colorize;
use mlua::{FromLua, Function, IntoLua, Lua, Value};
use utilities::language::*;
use utilities::DICTIONARY_CONFIG;
use wiktionary_en_entities::wiktionary_entity::*;

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

    pub fn intercept(dictionary_entry: &DictionaryEntry) -> Result<DictionaryEntry> {
        match intercept(dictionary_entry) {
            Ok(entry) => Ok(entry),
            Err(err) => {
                bail!("{}", err.to_string());
            }
        }
    }
}

pub fn intercept_witkionary_result(result: &Vec<DictionaryEntry>) -> Result<Vec<DictionaryEntry>> {
    let mut intercepted_result = Vec::new();
    for entry in result {
        intercepted_result.push(Config::intercept(&entry)?);
    }
    return Ok(intercepted_result);
}

fn intercept(dictionary_entry: &DictionaryEntry) -> mlua::Result<DictionaryEntry> {
    let lua = Lua::new();
    lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?)
        .exec()?;

    let apply_color_fn = apply_color(&lua)?;
    lua.globals().set("apply_color", apply_color_fn)?;
    let apply_style_fn = apply_style(&lua)?;
    lua.globals().set("apply_style", apply_style_fn)?;

    let intercept: mlua::Value = lua.globals().get("intercept")?;
    if let Some(intercept) = intercept.as_function() {
        return intercept.call(dictionary_entry.clone());
    }

    return Ok(dictionary_entry.clone());
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
            //"clear" => text.clear().to_string(),
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

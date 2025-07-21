use anyhow::{anyhow, Result};
use colored::Colorize;
use mlua::{FromLua, Function, Lua};
use utilities::colored_string_utils;
use utilities::LUA_DIR;
use utilities::{DICTIONARY_CONFIG, DICTIONARY_EXTENSIONS};
use wiktionary_en_db::client::DbClientMutex;
use wiktionary_en_entities::config::Config;
use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
use wiktionary_en_entities::history_entry::HistoryEntry;
use wiktionary_en_entities::result::{DidYouMean, SearchResult};

const LUA_CONFIGURATION_ERROR: &str = "Lua Configuration Error";
const LUA_EXTENSION_ERROR: &str = "Lua Extension Error";

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

pub struct ExtensionHandler {
    lua: Lua,
}

impl ExtensionHandler {
    pub fn init(db_client: DbClientMutex) -> Result<Self> {
        let lua = Lua::new();
        match create_importable_lua_module(&lua, "wiktionary_db_client", db_client) {
            Ok(_) => match init_lua(&lua) {
                Ok(_) => Ok(Self { lua }),
                Err(err) => Err(anyhow!("{}", err).context(LUA_EXTENSION_ERROR)),
            },
            Err(err) => Err(anyhow!("{}", err).context(LUA_EXTENSION_ERROR)),
        }
    }

    fn intercept(&self, dictionary_entry: &SearchResult) -> Result<Option<SearchResult>> {
        match intercept(&self.lua, dictionary_entry) {
            Ok(entry) => Ok(entry),
            Err(err) => Err(anyhow!("{}", err).context(LUA_EXTENSION_ERROR)),
        }
    }

    pub fn intercept_wiktionary_result(&self, wiktionary_result: &mut SearchResult) -> Result<()> {
        if let Some(intercepted_result) = self.intercept(wiktionary_result)? {
            *wiktionary_result = intercepted_result;
        } else {
            return Ok(());
        }

        Ok(())
    }

    fn format_entry(&self, dictionary_entry: &DictionaryEntry) -> Result<Option<String>> {
        match format_entry(&self.lua, dictionary_entry) {
            Ok(entry) => Ok(entry),
            Err(err) => Err(anyhow!("{}", err).context(LUA_EXTENSION_ERROR)),
        }
    }

    pub fn format_wiktionary_entries(
        &self,
        result: &[DictionaryEntry],
    ) -> Result<Option<Vec<String>>> {
        let mut formatted_entries = Vec::new();
        for entry in result {
            if let Some(formatted_entry) = self.format_entry(entry)? {
                formatted_entries.push(formatted_entry);
            } else {
                return Ok(None);
            }
        }
        Ok(Some(formatted_entries))
    }

    pub fn format_wiktionary_did_you_mean_banner(
        &self,
        did_you_mean: &DidYouMean,
    ) -> Result<Option<String>> {
        match format_did_you_mean_banner(&self.lua, did_you_mean) {
            Ok(result) => Ok(result),
            Err(err) => Err(anyhow!("{}", err).context(LUA_EXTENSION_ERROR)),
        }
    }

    pub fn format_history_entries(
        &self,
        history_entries: &[HistoryEntry],
    ) -> Result<Option<Vec<String>>> {
        let mut formatted_entries = Vec::new();
        for entry in history_entries {
            if let Some(formatted_entry) = self.format_history_entry(entry)? {
                formatted_entries.push(formatted_entry);
            } else {
                return Ok(None);
            }
        }
        Ok(Some(formatted_entries))
    }
    fn format_history_entry(&self, history_entry: &HistoryEntry) -> Result<Option<String>> {
        match format_history_entry(&self.lua, history_entry) {
            Ok(result) => Ok(result),
            Err(err) => Err(anyhow!("{}", err).context(LUA_EXTENSION_ERROR)),
        }
    }
}

fn init_lua(lua: &Lua) -> mlua::Result<()> {
    load_lua_api(lua)?;
    add_lua_library_to_path(lua)?;
    init_lua_exentions(lua)
}

fn init_lua_exentions(lua: &Lua) -> mlua::Result<()> {
    lua.load(std::fs::read_to_string(DICTIONARY_EXTENSIONS!())?)
        .exec()
}

fn init_lua_config(lua: &Lua) -> mlua::Result<()> {
    lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?)
        .exec()
}

fn create_importable_lua_module(
    lua: &Lua,
    package_name: &str,
    module: impl mlua::IntoLua,
) -> mlua::Result<()> {
    let package: mlua::Table = lua.globals().get("package")?;
    let loaded: mlua::Table = package.get("loaded")?;
    loaded.set(package_name, module)
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
    let project_folder_fn = project_folder(lua)?;
    wiktionary_api.set("project_folder", project_folder_fn)?;

    create_importable_lua_module(lua, "wiktionary_api", wiktionary_api)
}

fn add_lua_library_to_path(lua: &Lua) -> mlua::Result<()> {
    let package: mlua::Table = lua.globals().get("package")?;
    let path: String = package.get("path")?;
    package.set("path", format!("{};{}/?.lua", path, LUA_DIR!()))
}

fn get_config_as_lua_value(lua: &Lua) -> mlua::Result<mlua::Value> {
    lua.globals().get("config")
}

fn get_extensions_as_lua_value(lua: &Lua) -> mlua::Result<mlua::Value> {
    lua.globals().get("extensions")
}

fn get_config(lua: &Lua) -> mlua::Result<Config> {
    let config: mlua::Value = get_config_as_lua_value(lua)?;
    if let Some(config) = config.as_function() {
        return config.call(());
    }
    Config::from_lua(config, lua)
}

fn call_extension_lua_function<A, B>(
    lua: &Lua,
    function_name: &str,
    argument: &A,
) -> mlua::Result<Option<B>>
where
    A: mlua::IntoLuaMulti + Clone,
    B: mlua::FromLua,
{
    if let Some(config) = get_extensions_as_lua_value(lua)?.as_table() {
        let function: mlua::Value = config.get(function_name)?;
        if let Some(function) = function.as_function() {
            return Ok(Some(function.call(argument.clone())?));
        }
    }

    Ok(None)
}

fn intercept(lua: &Lua, wiktionary_result: &SearchResult) -> mlua::Result<Option<SearchResult>> {
    call_extension_lua_function(lua, "intercept", wiktionary_result)
}

fn format_entry(lua: &Lua, dictionary_entry: &DictionaryEntry) -> mlua::Result<Option<String>> {
    call_extension_lua_function(lua, "format_entry", dictionary_entry)
}

fn format_did_you_mean_banner(
    lua: &Lua,
    did_you_mean: &DidYouMean,
) -> mlua::Result<Option<String>> {
    call_extension_lua_function(lua, "format_did_you_mean_banner", did_you_mean)
}

fn format_history_entry(lua: &Lua, history_entry: &HistoryEntry) -> mlua::Result<Option<String>> {
    call_extension_lua_function(lua, "format_history_entry", history_entry)
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

fn project_folder(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(|_, ()| Ok(LUA_DIR!()))
}

use mlua::Lua;
use mlua::Value;
use mlua::{FromLua, IntoLua};

use crate::wiktionary_entity::{DictionaryEntry, Example, Sense, Sound, Translation};

impl FromLua for DictionaryEntry {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        if let Some(dictionary_entry) = value.as_table() {
            let entry = DictionaryEntry {
                lang_code: dictionary_entry.get("lang_code")?,
                word: dictionary_entry.get("word")?,
                senses: Vec::new(),
                pos: dictionary_entry.get("pos")?,
                translations: dictionary_entry.get("translations")?,
                sounds: Vec::new(),
                etymology_text: None,
            };
            return Ok(entry);
        }
        return Err(mlua::Error::RuntimeError(
            "no dictionary entry value found in lua".to_string(),
        ));
    }
}

impl FromLua for Translation {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        if let Some(translation) = value.as_table() {
            let entry = Translation {
                lang: translation.get("lang")?,
                code: translation.get("code")?,
                word: translation.get("word")?,
            };
            return Ok(entry);
        }
        return Err(mlua::Error::RuntimeError(
            "no translation array found in lua".to_string(),
        ));
    }
}

impl IntoLua for DictionaryEntry {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let dictionary_entry = lua.create_table()?;
        dictionary_entry.set("word", self.word)?;
        dictionary_entry.set("pos", self.pos)?;
        dictionary_entry.set("lang_code", self.lang_code)?;
        dictionary_entry.set("etymology", self.etymology_text)?;
        dictionary_entry.set("translations", self.translations)?;
        dictionary_entry.set("senses", self.senses)?;
        dictionary_entry.set("sounds", self.sounds)?;
        return Ok(mlua::Value::Table(dictionary_entry));
    }
}

impl IntoLua for Translation {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let translation = lua.create_table()?;
        translation.set("lang", self.lang)?;
        translation.set("code", self.code)?;
        translation.set("word", self.word)?;
        return Ok(mlua::Value::Table(translation));
    }
}

impl IntoLua for Sense {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let senses = lua.create_table()?;
        senses.set("glosses", self.glosses)?;
        senses.set("examples", self.examples)?;
        senses.set("tags", self.tags)?;
        return Ok(mlua::Value::Table(senses));
    }
}

impl IntoLua for Example {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let examples = lua.create_table()?;
        examples.set("reference", self.reference)?;
        examples.set("text", self.text)?;
        return Ok(mlua::Value::Table(examples));
    }
}

impl IntoLua for Sound {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let sounds = lua.create_table()?;
        sounds.set("ipa", self.ipa)?;
        sounds.set("enpr", self.enpr)?;
        sounds.set("tags", self.tags)?;
        return Ok(mlua::Value::Table(sounds));
    }
}

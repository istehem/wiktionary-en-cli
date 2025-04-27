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
                senses: dictionary_entry.get("senses")?,
                pos: dictionary_entry.get("pos")?,
                translations: dictionary_entry.get("translations")?,
                sounds: dictionary_entry.get("sounds")?,
                etymology_text: dictionary_entry.get("etymology")?,
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
            "no translation found in lua".to_string(),
        ));
    }
}

impl FromLua for Sense {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        if let Some(sense) = value.as_table() {
            let entry = Sense {
                glosses: sense.get("glosses")?,
                examples: sense.get("examples")?,
                tags: sense.get("tags")?,
            };
            return Ok(entry);
        }
        return Err(mlua::Error::RuntimeError(
            "no sense found in lua".to_string(),
        ));
    }
}

impl FromLua for Example {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        if let Some(sound) = value.as_table() {
            let entry = Example {
                reference: sound.get("reference")?,
                text: sound.get("text")?,
            };
            return Ok(entry);
        }
        return Err(mlua::Error::RuntimeError(
            "no example found in lua".to_string(),
        ));
    }
}

impl FromLua for Sound {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        if let Some(sound) = value.as_table() {
            let entry = Sound {
                ipa: sound.get("ipa")?,
                enpr: sound.get("enpr")?,
                tags: sound.get("tags")?,
            };
            return Ok(entry);
        }
        return Err(mlua::Error::RuntimeError(
            "no sound found in lua".to_string(),
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

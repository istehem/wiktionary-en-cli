use crate::dictionary_entry::DictionaryEntry;
use crate::history_entry::HistoryEntry;
use colored::Colorize;
use indoc::formatdoc;
use mlua::FromLua;
use mlua::IntoLua;
use mlua::Lua;
use mlua::Value;
use std::fmt;

pub struct HistoryResult {
    pub history_entries: Vec<HistoryEntry>,
}

#[derive(Clone)]
pub struct DidYouMean {
    pub searched_for: String,
    pub suggestion: String,
}

impl fmt::Display for DidYouMean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            &did_you_mean_banner(&self.searched_for, &self.suggestion)
        )
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

#[derive(Clone)]
pub struct SearchResult {
    pub word: String,
    pub did_you_mean: Option<DidYouMean>,
    pub hits: Vec<DictionaryEntry>,
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(did_you_mean) = &self.did_you_mean {
            writeln!(
                f,
                "{}",
                did_you_mean_banner(&did_you_mean.searched_for, &did_you_mean.suggestion)
            )?;
        }
        for hit in &self.hits {
            writeln!(f, "{}", &hit)?;
        }
        Ok(())
    }
}

fn did_you_mean_banner(search_term: &str, partial_match: &str) -> String {
    formatdoc!(
        "
        No result for {}.
        Did you mean  {}?
        ",
        search_term.red(),
        partial_match.yellow()
    )
}

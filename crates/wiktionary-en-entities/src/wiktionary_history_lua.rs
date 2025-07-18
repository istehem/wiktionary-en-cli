use mlua::IntoLua;
use mlua::Lua;
use mlua::Value;

use crate::wiktionary_history::HistoryEntry;

impl IntoLua for HistoryEntry {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let history_entry = lua.create_table()?;
        history_entry.set("word", self.word)?;
        history_entry.set("timestamp", self.last_hit.timestamp())?;
        Ok(mlua::Value::Table(history_entry))
    }
}

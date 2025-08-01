use mlua::IntoLua;
use mlua::Lua;
use mlua::Value;

use crate::history_entry::HistoryEntry;

impl IntoLua for HistoryEntry {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let history_entry = lua.create_table()?;
        history_entry.set("word", self.word)?;
        history_entry.set("last_seen_at", self.last_seen_at.timestamp())?;
        history_entry.set("count", self.count)?;
        Ok(mlua::Value::Table(history_entry))
    }
}

use bson::Bson;
use mlua::{Error, IntoLua, Lua, Result, UserData, UserDataMethods};
use std::sync::MutexGuard;

use crate::client::{DbClient, DbClientMutex, WiktionaryDocument};

fn lock(db_client: &DbClientMutex) -> Result<MutexGuard<'_, DbClient>> {
    match db_client.client.lock() {
        Ok(db_client) => Ok(db_client),
        Err(err) => Err(Error::RuntimeError(err.to_string())),
    }
}

impl UserData for DbClientMutex {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("write_to_history", |_, this, word: String| {
            let db_client = lock(this)?;
            match db_client.upsert_into_history(&word) {
                Ok(_) => Ok(()),
                Err(err) => Err(Error::RuntimeError(err.to_string())),
            }
        });
        methods.add_method("find_in_history", |_, this, word: String| {
            let db_client = lock(this)?;
            match db_client.find_in_history_by_word(&word) {
                Ok(entry) => Ok(entry),
                Err(err) => Err(Error::RuntimeError(err.to_string())),
            }
        });

        methods.add_method("find_all_in_history", |_, this, _: ()| {
            let db_client = lock(this)?;
            match db_client.find_all_in_history() {
                Ok(entry) => Ok(entry),
                Err(err) => Err(Error::RuntimeError(err.to_string())),
            }
        });
    }
}

impl IntoLua for WiktionaryDocument {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;
        for (k, v) in self.document {
            table.set(k, bson_to_lua_value(v, lua)?)?;
        }
        Ok(mlua::Value::Table(table))
    }
}

fn bson_to_lua_value(bson: Bson, lua: &Lua) -> mlua::Result<mlua::Value> {
    match bson {
        Bson::String(s) => Ok(mlua::Value::String(lua.create_string(&s)?)),
        Bson::Int32(i) => Ok(mlua::Value::Integer(i64::from(i))),
        Bson::Int64(i) => Ok(mlua::Value::Integer(i)),
        Bson::Double(f) => Ok(mlua::Value::Number(f)),
        Bson::Boolean(b) => Ok(mlua::Value::Boolean(b)),
        Bson::Array(arr) => {
            let tbl = lua.create_table()?;
            for (i, item) in arr.into_iter().enumerate() {
                tbl.set(i + 1, bson_to_lua_value(item, lua)?)?; // 1-indexed
            }
            Ok(mlua::Value::Table(tbl))
        }
        Bson::Document(doc) => {
            let nested = WiktionaryDocument::from(doc).into_lua(lua)?;
            Ok(nested) // already a Value::Table
        }
        Bson::Null | Bson::Undefined => Ok(mlua::Value::Nil),
        _ => {
            Err(mlua::Error::FromLuaConversionError {
                from: "unmatched bson type",
                to: "table".to_string(),
                message: None,
            })
        }
    }
}

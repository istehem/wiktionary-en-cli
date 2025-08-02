use bson::Bson;
use bson::Document;
use mlua::{Error, FromLua, IntoLua, Lua, Result, Table, UserData, UserDataMethods, Value};
use std::any::type_name;
use std::sync::MutexGuard;

use crate::client::{DbClient, DbClientMutex, ExtensionDocument};

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
        methods.add_method(
            "find_in_collection",
            |_, this, (extension_name, document): (String, ExtensionDocument)| {
                let db_client = lock(this)?;
                match db_client.find_in_extension_collection(document) {
                    Ok(entry) => Ok(entry),
                    Err(err) => Err(Error::RuntimeError(err.to_string())),
                }
            },
        );
        methods.add_method(
            "find_one_in_collection",
            |_, this, (extension_name, document): (String, ExtensionDocument)| {
                let db_client = lock(this)?;
                match db_client.find_one_in_extension_collection(document) {
                    Ok(entry) => Ok(entry),
                    Err(err) => Err(Error::RuntimeError(err.to_string())),
                }
            },
        );
        methods.add_method(
            "insert_one_in_collection",
            |lua, this, (extension_name, document): (String, ExtensionDocument)| {
                let db_client = lock(this)?;
                match db_client.insert_one_into_extension_collection(&extension_name, document) {
                    Ok(result) => bson_to_lua_value(result, lua),
                    Err(err) => Err(Error::RuntimeError(err.to_string())),
                }
            },
        );
        methods.add_method(
            "update_one_in_collection",
            |_, this, (extension_name, query, update): (String, ExtensionDocument, ExtensionDocument)| {
                let db_client = lock(this)?;
                match db_client.update_one_in_extension_collection(&extension_name, query, update) {
                    Ok(update_count) => Ok(update_count),
                    Err(err) => Err(Error::RuntimeError(err.to_string())),
                }
            },
        );
    }
}

impl IntoLua for ExtensionDocument {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        let table = lua.create_table()?;
        for (k, v) in self.document {
            table.set(k, bson_to_lua_value(v, lua)?)?;
        }
        Ok(mlua::Value::Table(table))
    }
}

impl FromLua for ExtensionDocument {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        if let Some(lua_document) = value.as_table() {
            let mut document = Document::new();
            for pair in lua_document.pairs::<mlua::Value, mlua::Value>() {
                let (k, v) = pair?;
                let key = match k {
                    Value::String(s) => s.to_str()?.to_owned(),
                    _ => return Err(mlua::Error::runtime("bson document keys must be strings")),
                };
                let bson_value = lua_value_to_bson(v)?;
                document.insert(key, bson_value);
            }
            return Ok(ExtensionDocument::from(document));
        }
        Err(mlua::Error::FromLuaConversionError {
            from: "value",
            to: type_name::<Self>().to_string(),
            message: None,
        })
    }
}

fn lua_value_to_bson(value: Value) -> mlua::Result<Bson> {
    match value {
        Value::Nil => Ok(bson::Bson::Null),
        Value::Boolean(b) => Ok(bson::Bson::Boolean(b)),
        Value::Integer(i) => {
            Ok(bson::Bson::Int32(i.try_into().map_err(|_| {
                mlua::Error::runtime("integer overflow for i32")
            })?))
        }
        Value::Number(n) => Ok(bson::Bson::Double(n)),
        Value::String(s) => Ok(bson::Bson::String(s.to_str()?.to_owned())),
        Value::Table(table) => {
            if is_array(&table)? {
                let mut vec = Vec::new();
                for value in table.sequence_values::<Value>() {
                    vec.push(lua_value_to_bson(value?)?);
                }
                Ok(bson::Bson::Array(vec))
            } else {
                let mut doc = Document::new();
                for pair in table.pairs::<mlua::Value, mlua::Value>() {
                    let (k, v) = pair?;
                    let key = match k {
                        Value::String(s) => s.to_str()?.to_owned(),
                        _ => {
                            return Err(mlua::Error::runtime("bson document keys must be strings"))
                        }
                    };
                    doc.insert(key, lua_value_to_bson(v)?);
                }
                Ok(bson::Bson::Document(doc))
            }
        }
        _ => Err(mlua::Error::FromLuaConversionError {
            from: "value",
            to: type_name::<Bson>().to_string(),
            message: None,
        }),
    }
}

fn is_array(table: &Table) -> mlua::Result<bool> {
    let len = table.len()?;
    for i in 1..=len {
        if !table.contains_key(i)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn bson_to_lua_value(bson: Bson, lua: &Lua) -> mlua::Result<mlua::Value> {
    match bson {
        Bson::String(s) => Ok(mlua::Value::String(lua.create_string(&s)?)),
        Bson::Int32(i) => Ok(mlua::Value::Integer(i64::from(i))),
        Bson::Int64(i) => Ok(mlua::Value::Integer(i)),
        Bson::Double(f) => Ok(mlua::Value::Number(f)),
        Bson::Boolean(b) => Ok(mlua::Value::Boolean(b)),
        Bson::ObjectId(id) => Ok(mlua::Value::String(lua.create_string(id.to_hex())?)),
        Bson::Array(arr) => {
            let tbl = lua.create_table()?;
            for (i, item) in arr.into_iter().enumerate() {
                tbl.set(i + 1, bson_to_lua_value(item, lua)?)?; // 1-indexed
            }
            Ok(mlua::Value::Table(tbl))
        }
        Bson::Document(doc) => {
            let nested = ExtensionDocument::from(doc).into_lua(lua)?;
            Ok(nested)
        }
        Bson::Null | Bson::Undefined => Ok(mlua::Value::Nil),
        other => Err(mlua::Error::ToLuaConversionError {
            from: other.to_string(),
            to: "table",
            message: Some("conversion for this bson type isn't implemented yet".to_string()),
        }),
    }
}

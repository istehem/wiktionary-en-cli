use anyhow;
use mlua::{Error, FromLua, IntoLua, Lua, LuaSerdeExt, Result, UserData, UserDataMethods, Value};

use crate::client::{DbClientMutex, Document};

fn ok_or_runtime_error<T>(result: anyhow::Result<T>) -> Result<T> {
    result.map_err(|err| Error::RuntimeError(err.to_string()))
}

impl UserData for DbClientMutex {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method(
            "find_in_collection",
            async |_, this, (extension_name, document): (String, Document)| {
                let db_client = &this.client.lock().await;
                ok_or_runtime_error(
                    db_client
                        .find_in_extension_collection(&extension_name, document)
                        .await,
                )
            },
        );
        methods.add_async_method(
            "find_one_in_collection",
            async |_, this, (extension_name, document): (String, Document)| {
                let db_client = &this.client.lock().await;
                ok_or_runtime_error(
                    db_client
                        .find_one_in_extension_collection(&extension_name, document)
                        .await,
                )
            },
        );
        methods.add_async_method(
            "insert_one_into_collection",
            async |_, this, (extension_name, document): (String, Document)| {
                let db_client = &this.client.lock().await;
                ok_or_runtime_error(
                    db_client
                        .insert_one_into_extension_collection(&extension_name, document)
                        .await,
                )
            },
        );
        methods.add_async_method(
            "update_one_in_collection",
            async |_, this, (extension_name, document): (String, Document)| {
                let db_client = &this.client.lock().await;
                ok_or_runtime_error(
                    db_client
                        .update_one_in_extension_collection(&extension_name, document)
                        .await,
                )
            },
        );
        methods.add_async_method(
            "delete_in_collection",
            async |_, this, (extension_name, query): (String, Document)| {
                let db_client = &this.client.lock().await;
                ok_or_runtime_error(
                    db_client
                        .delete_many_in_extension_collection(&extension_name, query)
                        .await,
                )
            },
        );
        methods.add_async_method(
            "get_view_in_collection",
            async |_, this, (extension_name, view): (String, Document)| {
                let db_client = &this.client.lock().await;
                ok_or_runtime_error(
                    db_client
                        .get_view_in_extension_collection(&extension_name, view)
                        .await,
                )
            },
        );
        methods.add_async_method(
            "create_index_for_collection",
            async |_, this, (extension_name, keys): (String, Document)| {
                let db_client = &this.client.lock().await;
                ok_or_runtime_error(
                    db_client
                        .create_index_for_extension_collection(&extension_name, keys)
                        .await,
                )
            },
        );
        methods.add_async_method(
            "create_view_in_collection",
            async |_, this, (extension_name, definition): (String, Document)| {
                let db_client = &this.client.lock().await;
                ok_or_runtime_error(
                    db_client
                        .create_view_in_extension_collection(&extension_name, definition)
                        .await,
                )
            },
        );
    }
}

impl IntoLua for Document {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        lua.to_value(&self.document)
    }
}

impl FromLua for Document {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        Ok(Document::from(lua.from_value(value)?))
    }
}

use mlua::Result;
use std::sync::MutexGuard;

use crate::client::{DbClient, DbClientMutex};

fn lock(db_client: &DbClientMutex) -> Result<MutexGuard<'_, DbClient>> {
    match db_client.client.lock() {
        Ok(db_client) => Ok(db_client),
        Err(err) => Err(mlua::Error::RuntimeError(err.to_string())),
    }
}

impl mlua::UserData for DbClientMutex {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("write_to_history", |_, this, word: String| {
            let db_client = lock(this)?;
            match db_client.upsert_into_history(&word) {
                Ok(_) => Ok(()),
                Err(err) => Err(mlua::Error::RuntimeError(err.to_string())),
            }
        });
        methods.add_method("find_in_history", |_, this, word: String| {
            let db_client = lock(this)?;
            match db_client.find_in_history_by_word(&word) {
                Ok(entry) => Ok(entry),
                Err(err) => Err(mlua::Error::RuntimeError(err.to_string())),
            }
        });

        methods.add_method("find_all_in_history", |_, this, _: ()| {
            let db_client = lock(this)?;
            match db_client.find_all_in_history() {
                Ok(entry) => Ok(entry),
                Err(err) => Err(mlua::Error::RuntimeError(err.to_string())),
            }
        });
    }
}

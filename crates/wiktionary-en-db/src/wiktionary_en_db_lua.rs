use mlua::Result;
use std::sync::MutexGuard;

use crate::wiktionary_en_db::{WiktionaryDbClient, WiktionaryDbClientMutex};

fn lock<'a>(db_client: &'a WiktionaryDbClientMutex) -> Result<MutexGuard<'a, WiktionaryDbClient>> {
    match db_client.client.lock() {
        Ok(db_client) => Ok(db_client),
        Err(err) => Err(mlua::Error::RuntimeError(String::from(err.to_string()))),
    }
}

impl mlua::UserData for WiktionaryDbClientMutex {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("write_to_history", |_, this, word: String| {
            let db_client = lock(this)?;
            match db_client.upsert_into_history(&word) {
                Ok(_) => Ok(()),
                Err(err) => Err(mlua::Error::RuntimeError(err.to_string())),
            }
        });
    }
}

use mlua;
use std::sync::{Arc, Mutex};

use crate::wiktionary_en_db::WiktionaryDbClient;

pub struct WiktionaryDbClientWrapper {
    pub client: Arc<Mutex<WiktionaryDbClient>>,
}

impl mlua::UserData for WiktionaryDbClientWrapper {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("write_to_history", |_, this, word: String| {
            let client = this.client.lock().unwrap();
            match client.upsert_into_history(&word) {
                Ok(_) => Ok(()),
                Err(err) => Err(mlua::Error::RuntimeError(err.to_string())),
            }
        });
    }
}

impl mlua::UserData for WiktionaryDbClient {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("write_to_history", |_, this, word: String| {
            match this.upsert_into_history(&word) {
                Ok(_) => Ok(()),
                Err(err) => Err(mlua::Error::RuntimeError(err.to_string())),
            }
        });
    }
}

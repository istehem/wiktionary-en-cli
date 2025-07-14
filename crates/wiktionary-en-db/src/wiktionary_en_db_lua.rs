use mlua;

use crate::wiktionary_en_db::WiktionaryDbClientMutex;

impl mlua::UserData for WiktionaryDbClientMutex {
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

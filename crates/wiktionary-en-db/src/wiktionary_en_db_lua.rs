use mlua;

use crate::wiktionary_en_db::WiktionaryDbClient;

/*
// Wrap in Arc and Mutex to allow shared mutable access
let db = Arc::new(Mutex::new(DbConnection::new()));

// Set the db object as a global Lua value
lua.globals().set("db", db)?;
*/

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

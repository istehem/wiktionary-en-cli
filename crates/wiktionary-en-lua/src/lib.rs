use anyhow::{bail, Result};
use mlua::Lua;
use utilities::DICTIONARY_CONFIG;

fn load_config(lua: &Lua) -> mlua::Result<u8> {
    let result: u8 = lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?).eval()?;
    println!("lua returns the result: {}", result);
    return Ok(result);
}

pub fn do_one_plus_one() -> Result<u8> {
    let lua = Lua::new();
    match load_config(&lua) {
        Ok(result) => return Ok(result),
        Err(err) => bail!(err.to_string()),
    }
}

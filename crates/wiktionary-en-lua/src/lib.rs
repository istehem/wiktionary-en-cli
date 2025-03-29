use anyhow::{bail, Result};
use mlua::Lua;
use utilities::DICTIONARY_CONFIG;

fn load_config(lua: &Lua) -> mlua::Result<u8> {
    lua.load(std::fs::read_to_string(DICTIONARY_CONFIG!())?)
        .exec()?;
    let one_plus_one: mlua::Function = lua.globals().get("one_plus_one")?;
    let result: u8 = one_plus_one.call(())?;
    println!("lua function returned: {}", result);
    return Ok(result);
}

pub fn do_one_plus_one() -> Result<u8> {
    let lua = Lua::new();
    match load_config(&lua) {
        Ok(result) => return Ok(result),
        Err(err) => bail!(err.to_string()),
    }
}

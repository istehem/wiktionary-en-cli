use anyhow::{bail, Result};
use mlua::Lua;

fn on_plus_one(lua: &Lua) -> mlua::Result<u8> {
    let result: u8 = lua.load("return 1 + 1").eval()?;
    println!("lua returns the result: {}", result);
    return Ok(result);
}

pub fn do_one_plus_one() -> Result<u8> {
    let lua = Lua::new();
    match on_plus_one(&lua) {
        Ok(result) => return Ok(result),
        Err(err) => bail!(err.to_string()),
    }
}

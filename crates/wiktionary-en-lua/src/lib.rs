use anyhow::{bail, Result};
use mlua::Lua;

fn on_plus_one(lua: &Lua) -> mlua::Result<u8> {
    let result: u8 = lua.load("return 1 + 1").eval()?;
    println!("lua return the result: {}", result);
    return Ok(result);
}

pub fn do_one_plus_one() -> Result<u8> {
    let lua = Lua::new();
    if let Ok(result) = on_plus_one(&lua) {
        return Ok(result);
    }
    bail!("could not execute the lua fuction");
}

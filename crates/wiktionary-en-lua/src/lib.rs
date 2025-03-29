use anyhow::Result;
use rlua::{Function, Lua};

fn rust_function(lua: &Lua) -> rlua::Result<Function> {
    lua.create_function(|_, arg: String| Ok(format!("Rust received: {}", arg)))
}

pub fn hello() -> Result<()> {
    let lua = Lua::new();
    let func = rust_function(&lua)?;
    lua.globals().set("rustFunc", func)?;

    lua.load(
        r#"
        print(rustFunc("Hello from Lua!"))
    "#,
    )
    .exec()?;
    Ok(())
}

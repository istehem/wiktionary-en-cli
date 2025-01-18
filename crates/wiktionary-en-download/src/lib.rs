use anyhow::{Result};
use std::fs::File;
use std::io::Write;

pub fn download() -> Result<()> {
    //https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl

    let response = reqwest::blocking::get("https://thewowstyle.com/wp-content/uploads/2015/01/free-beautiful-place-wallpaper-hd-173.jpg")?
        .bytes()?;

    let mut f = File::create("remove-me.jpg")?;
    f.write_all(&response)?;
    return Ok(());
}



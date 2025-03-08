use anyhow::{bail, Result};
use minus::Pager;
use std::fmt::Display;
use std::fmt::Write;
use streaming_iterator::*;

pub fn print_lines_in_pager<T: Display>(entries: &Vec<T>) -> Result<()> {
    let mut output = Pager::new();

    for entry in entries {
        writeln!(output, "{}", entry)?;
    }
    minus::page_all(output)?;
    return Ok(());
}

pub fn print_in_pager<T: std::fmt::Display>(value: &T) -> Result<()> {
    let mut output = Pager::new();
    writeln!(output, "{}", value)?;
    minus::page_all(output)?;
    return Ok(());
}

pub fn print_in_existing_pager<T: std::fmt::Display>(pager: &mut Pager, value: &T) -> Result<()> {
    writeln!(pager, "{}", value)?;
    minus::page_all(pager.clone())?;
    return Ok(());
}

pub fn print_stream_in_pager<X: std::fmt::Display>(
    mut iterator: impl StreamingIterator<Item = Result<Option<X>>>,
) -> Result<()> {
    let mut output = Pager::new();
    while let Some(item) = iterator.next() {
        if let Ok(Some(item)) = item {
            writeln!(output, "{}", item)?;
        } else if let Err(err) = item {
            // “{:?}” includes a backtrace if one was captured
            bail!(format!("{:?}", err));
        }
        minus::page_all(output.clone())?;
    }

    return Ok(());
}

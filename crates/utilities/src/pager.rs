use anyhow::Result;
use minus::Pager;
use std::fmt::Display;
use std::fmt::Write;

pub fn print_lines_in_pager<T: Display>(entries: &[T]) -> Result<()> {
    let mut output = Pager::new();

    for entry in entries {
        writeln!(output, "{}", entry)?;
    }
    minus::page_all(output)?;
    Ok(())
}

pub fn print_in_pager<T: std::fmt::Display>(value: &T) -> Result<()> {
    let mut output = Pager::new();
    writeln!(output, "{}", value)?;
    minus::page_all(output)?;
    Ok(())
}

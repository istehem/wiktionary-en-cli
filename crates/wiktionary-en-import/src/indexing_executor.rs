use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressStyle};
use streaming_iterator::*;

const PROGRESS_BAR_TEMPLATE: &str = "{spinner} {elapsed}\nLatest error: {msg}";
const NO_LATEST_ERROR: &str = "<None>";

pub fn execute_with_progress_bar_and_message<X: std::fmt::Display>(
    mut iterator: impl StreamingIterator<Item = Result<Option<X>>>,
) -> Result<Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    let progress_bar = ProgressBar::no_length()
        .with_message(NO_LATEST_ERROR)
        .with_style(ProgressStyle::default_spinner().template(PROGRESS_BAR_TEMPLATE)?);
    while let Some(item) = iterator.next() {
        if let Ok(Some(item)) = item {
            let error_message = format!("{}", item);
            errors.push(error_message.clone());
            progress_bar.set_message(error_message);
        } else if let Err(err) = item {
            // “{:?}” includes a backtrace if one was captured
            bail!(format!("{:?}", err));
        }
        progress_bar.tick();
    }
    progress_bar.finish();
    return Ok(errors);
}

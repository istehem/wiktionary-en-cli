use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressStyle};
use streaming_iterator::StreamingIterator;

const PROGRESS_BAR_TEMPLATE: &str = "{spinner} {elapsed}{msg}";
const EMPTY_STRING: &str = "";

pub fn execute_with_progress_bar_and_message<X: std::fmt::Display>(
    mut iterator: impl StreamingIterator<Item = Result<Option<X>>>,
) -> Result<Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    let progress_bar = ProgressBar::no_length()
        .with_style(ProgressStyle::default_spinner().template(PROGRESS_BAR_TEMPLATE)?);
    while let Some(item) = iterator.next() {
        if let Ok(Some(item)) = item {
            let error_message = format!("{}", item);
            let progress_bar_message = format!("\n Latest Error: {}", error_message);
            errors.push(error_message.clone());
            progress_bar.set_message(progress_bar_message);
        } else if let Err(err) = item {
            // “{:?}” includes a backtrace if one was captured
            bail!(format!("{:?}", err));
        }
        progress_bar.tick();
    }
    progress_bar.set_message(EMPTY_STRING);
    progress_bar.finish();
    Ok(errors)
}

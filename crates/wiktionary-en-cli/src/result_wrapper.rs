use anyhow::{bail, Result};
use std::fmt;

use wiktionary_en_entities::wiktionary_history::HistoryEntry;
use wiktionary_en_entities::wiktionary_result::*;
use wiktionary_en_lua::ExtensionHandler;

pub struct HistoryResult {
    pub history_entries: Vec<HistoryEntry>,
}

pub enum WiktionaryResult2 {
    HistoryResult(HistoryResult),
    SearchResult(WiktionaryResult),
}

pub struct WiktionaryResultWrapper {
    pub result: WiktionaryResult2,
    pub extension_handler: wiktionary_en_lua::ExtensionHandler,
}

impl fmt::Display for WiktionaryResultWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.result {
            WiktionaryResult2::HistoryResult(result) => {
                fmt_history_result(f, &self.extension_handler, result)
            }
            WiktionaryResult2::SearchResult(result) => {
                fmt_wiktionary_result(f, &self.extension_handler, &result)
            }
        }
    }
}

impl WiktionaryResultWrapper {
    pub fn intercept(&mut self) -> Result<()> {
        match &mut self.result {
            WiktionaryResult2::HistoryResult(_) => bail!("nothing to intercept"),
            WiktionaryResult2::SearchResult(result) => {
                self.extension_handler.intercept_wiktionary_result(result)
            }
        }
    }
}

fn fmt_wiktionary_result(
    f: &mut fmt::Formatter<'_>,
    extension_handler: &ExtensionHandler,
    wiktionary_result: &WiktionaryResult,
) -> fmt::Result {
    if let Some(did_you_mean) = &wiktionary_result.did_you_mean {
        match extension_handler.format_wiktionary_did_you_mean_banner(&did_you_mean) {
            Ok(Some(formatted_banner)) => {
                writeln!(f, "{}", &formatted_banner)?;
            }
            Ok(None) => {
                writeln!(f, "{}", &did_you_mean)?;
            }
            Err(err) => {
                writeln!(f, "{:?}", err)?;
                return Err(fmt::Error);
            }
        }
    }

    match extension_handler.format_wiktionary_entries(&wiktionary_result.hits) {
        Ok(Some(formated_hits)) => {
            for hit in &formated_hits {
                writeln!(f, "{}", &hit)?;
            }
            Ok(())
        }
        Ok(None) => {
            for hit in &wiktionary_result.hits {
                writeln!(f, "{}", &hit)?;
            }
            Ok(())
        }
        Err(err) => {
            writeln!(f, "{:?}", err)?;
            Err(fmt::Error)
        }
    }
}

fn fmt_history_result(
    f: &mut fmt::Formatter<'_>,
    extension_handler: &ExtensionHandler,
    history_result: &HistoryResult,
) -> fmt::Result {
    match extension_handler.format_history_entries(&history_result.history_entries) {
        Ok(Some(formated_hits)) => {
            for hit in &formated_hits {
                writeln!(f, "{}", &hit)?;
            }
            Ok(())
        }
        Ok(None) => {
            for hit in &history_result.history_entries {
                writeln!(f, "{}", &hit)?;
            }
            Ok(())
        }
        Err(err) => {
            writeln!(f, "{:?}", err)?;
            Err(fmt::Error)
        }
    }
}

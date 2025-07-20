use anyhow::{bail, Result};

use wiktionary_en_entities::wiktionary_result::{HistoryResult, SearchResult};
use wiktionary_en_lua::ExtensionHandler;

pub enum WiktionaryResult {
    HistoryResult(HistoryResult),
    SearchResult(SearchResult),
}

pub struct WiktionaryResultWrapper {
    pub result: WiktionaryResult,
    pub extension_handler: wiktionary_en_lua::ExtensionHandler,
}

impl WiktionaryResultWrapper {
    pub fn intercept(&mut self) -> Result<()> {
        match &mut self.result {
            WiktionaryResult::HistoryResult(_) => bail!("nothing to intercept"),
            WiktionaryResult::SearchResult(result) => {
                self.extension_handler.intercept_wiktionary_result(result)
            }
        }
    }

    pub fn fmt(&self) -> Result<String> {
        match &self.result {
            WiktionaryResult::HistoryResult(result) => {
                fmt_history_result(&self.extension_handler, result)
            }
            WiktionaryResult::SearchResult(result) => {
                fmt_wiktionary_result(&self.extension_handler, result)
            }
        }
    }
}

fn fmt_wiktionary_result(
    extension_handler: &ExtensionHandler,
    wiktionary_result: &SearchResult,
) -> Result<String> {
    let mut formatted = Vec::new();
    if let Some(did_you_mean) = &wiktionary_result.did_you_mean {
        match extension_handler.format_wiktionary_did_you_mean_banner(did_you_mean) {
            Ok(Some(formatted_banner)) => {
                formatted.push(formatted_banner.to_string());
            }
            Ok(None) => {
                formatted.push(did_you_mean.to_string());
            }
            Err(err) => {
                bail!(err);
            }
        }
    }

    match extension_handler.format_wiktionary_entries(&wiktionary_result.hits) {
        Ok(Some(formated_hits)) => {
            for hit in &formated_hits {
                formatted.push(hit.to_string());
            }
            Ok(formatted.join("\n"))
        }
        Ok(None) => {
            for entry in &wiktionary_result.hits {
                formatted.push(entry.to_string());
            }
            Ok(formatted.join("\n"))
        }
        Err(err) => {
            bail!(err)
        }
    }
}

fn fmt_history_result(
    extension_handler: &ExtensionHandler,
    history_result: &HistoryResult,
) -> Result<String> {
    match extension_handler.format_history_entries(&history_result.history_entries) {
        Ok(Some(formatted_entries)) => Ok(formatted_entries.join("\n")),
        Ok(None) => {
            let mut formatted_entries = Vec::new();
            for entry in &history_result.history_entries {
                formatted_entries.push(format!("{}", entry));
            }
            Ok(formatted_entries.join("\n"))
        }
        Err(err) => {
            bail!(err)
        }
    }
}

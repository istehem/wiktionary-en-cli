use anyhow::{bail, Result};

use wiktionary_en_entities::result::DictionaryResult;
use wiktionary_en_lua::extension::ExtensionHandler;

pub enum WiktionaryResult {
    DictionaryResult(DictionaryResult),
}

pub struct WiktionaryResultWrapper {
    pub result: WiktionaryResult,
    pub extension_handler: ExtensionHandler,
}

impl WiktionaryResultWrapper {
    pub fn intercept(&mut self) -> Result<()> {
        match &mut self.result {
            WiktionaryResult::DictionaryResult(result) => {
                self.extension_handler.intercept_dictionary_result(result)
            }
        }
    }

    pub fn fmt(&self) -> Result<String> {
        match &self.result {
            WiktionaryResult::DictionaryResult(result) => {
                fmt_dictionary_result(&self.extension_handler, result)
            }
        }
    }
}

fn fmt_dictionary_result(
    extension_handler: &ExtensionHandler,
    dictionary_result: &DictionaryResult,
) -> Result<String> {
    let mut formatted = Vec::new();
    if let Some(did_you_mean) = &dictionary_result.did_you_mean {
        match extension_handler.format_dictionary_did_you_mean_banner(did_you_mean) {
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

    match extension_handler.format_dictionary_entries(&dictionary_result.hits) {
        Ok(Some(formated_hits)) => {
            for hit in &formated_hits {
                formatted.push(hit.to_string());
            }
            Ok(formatted.join("\n"))
        }
        Ok(None) => {
            for entry in &dictionary_result.hits {
                formatted.push(entry.to_string());
            }
            Ok(formatted.join("\n"))
        }
        Err(err) => {
            bail!(err)
        }
    }
}

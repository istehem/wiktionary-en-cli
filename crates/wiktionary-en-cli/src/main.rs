use anyhow::{bail, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use utilities::language::*;

use wiktionary_en_entities::wiktionary_entry::*;
use wiktionary_en_entities::wiktionary_result::*;

use wiktionary_en_db::wiktionary_en_db::*;

mod wiktionary_stats;
use crate::wiktionary_stats::*;

mod exhaustive_search;


use std::fmt;

/// A To English Dictionary
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Override dictionary db file to use
    #[clap(long, short = 'd')]
    db_path: Option<String>,
    /// A word to search for; omitting it will yield a random entry
    search_term: Option<String>,
    /// Maximal number of results
    #[clap(short, long, default_value = "1")]
    max_results: usize,
    /// Use case insensitive search
    #[clap(short = 'i', long)]
    case_insensitive: bool,
    /// Set search term language
    #[clap(long, short = 'l')]
    language: Option<String>,
    /// Show dictionary information
    #[clap(short, long)]
    stats: bool,
    #[cfg(feature = "sonic")]
    /// Autocomplete word
    #[clap(short, long)]
    autocomplete: bool,
    #[cfg(feature = "sonic")]
    /// Query word
    #[clap(short, long)]
    query: bool,
}

struct WiktionaryResultWrapper {
    result: WiktionaryResult,
    config_handler: wiktionary_en_lua::ConfigHandler,
}

impl WiktionaryResultWrapper {
    pub fn intercept(&mut self) -> Result<()> {
        if let Some(hits) = self
            .config_handler
            .intercept_wiktionary_result(&self.result.hits)?
        {
            self.result.hits = hits;
        }
        Ok(())
    }
}

impl fmt::Display for WiktionaryResultWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(did_you_mean) = &self.result.did_you_mean {
            match self
                .config_handler
                .format_wiktionary_did_you_mean_banner(did_you_mean)
            {
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

        match self
            .config_handler
            .format_wiktionary_entries(&self.result.hits)
        {
            Ok(Some(formated_hits)) => {
                for hit in &formated_hits {
                    writeln!(f, "{}", &hit)?;
                }
                Ok(())
            }
            Ok(None) => {
                for hit in &self.result.hits {
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
}

struct QueryParameters {
    search_term: Option<String>,
    language: Language,
    max_results: usize,
    case_insensitive: bool,
    path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedDbEntry {
    entries: Vec<DictionaryEntry>,
}

fn print_stats(input_path_buf: PathBuf, language: &Language) -> Result<()> {
    let input_path = input_path_buf.as_path();
    if input_path.is_dir() {
        bail!(
            "specified wiktionary extract file '{}' is a directory",
            input_path.display()
        );
    }

    println!(
        "{}",
        calculate_stats(input_path, language).to_pretty_string()
    );

    Ok(())
}

fn find_by_word_in_db(term: &String, language: &Language) -> Result<Option<Vec<DictionaryEntry>>> {
    let db_hits = find_by_word(term, language);
    match db_hits {
        Ok(result) => {
            if result.is_empty() {
                Ok(None)
            } else {
                Ok(Some(result))
            }
        }
        Err(err) => bail!(err),
    }
}

fn run(
    query_params: QueryParameters,
    config_handler: wiktionary_en_lua::ConfigHandler,
) -> Result<WiktionaryResultWrapper> {
    match query_params.search_term {
        Some(s) => match find_by_word_in_db(&s, &query_params.language) {
            Ok(Some(csr)) => {
                let result = WiktionaryResult {
                    did_you_mean: None,
                    hits: csr,
                };
                Ok(WiktionaryResultWrapper {
                    result,
                    config_handler,
                })
            }

            Ok(None) => {
                #[cfg(feature = "sonic")]
                {
                    let did_you_mean =
                        wiktionary_en_identifier_index::did_you_mean(&query_params.language, &s)?;
                    if let Some(did_you_mean) = did_you_mean {
                        let hits = find_by_word_in_db(&did_you_mean, &query_params.language)?;
                        if let Some(hits) = hits {
                            let result = WiktionaryResult {
                                did_you_mean: Some(DidYouMean {
                                    searched_for: s.to_string(),
                                    suggestion: did_you_mean,
                                }),
                                hits,
                            };
                            return Ok(WiktionaryResultWrapper {
                                result,
                                config_handler,
                            });
                        }
                    }
                }
                match exhaustive_search::search(
                    &query_params.path,
                    s.clone(),
                    query_params.max_results,
                    query_params.case_insensitive,
                ) {
                    Ok(sr) => {
                        if let Some(did_you_mean) = sr.did_you_mean {
                            let result = WiktionaryResult {
                                did_you_mean: Some(DidYouMean {
                                    searched_for: s.to_string(),
                                    suggestion: did_you_mean.word.clone(),
                                }),
                                hits: vec![did_you_mean],
                            };
                            return Ok(WiktionaryResultWrapper {
                                result,
                                config_handler,
                            });
                        }
                        let result = WiktionaryResult {
                            did_you_mean: None,
                            hits: sr.full_matches,
                        };
                        Ok(WiktionaryResultWrapper {
                            result,
                            config_handler,
                        })
                    }
                    Err(e) => bail!(e),
                }
            }
            Err(e) => bail!(e),
        },
        None => {
            let hit = random_entry_for_language(&query_params.language)?;
            let result = WiktionaryResult {
                did_you_mean: None,
                hits: vec![hit],
            };
            Ok(WiktionaryResultWrapper {
                result,
                config_handler,
            })
        }
    }
}

fn get_language(language: &Option<String>) -> Result<Option<Language>> {
    if let Some(language) = language {
        return Ok(Some(language.parse()?));
    }
    Ok(None)
}

fn get_db_path(path: Option<String>, language: &Language) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }
    PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()))
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language)?;
    let config_handler = wiktionary_en_lua::ConfigHandler::init()?;
    let language_to_use = language.unwrap_or(config_handler.config.language.unwrap_or_default());

    if args.stats {
        return print_stats(
            get_db_path(args.db_path, &language_to_use),
            &language_to_use,
        );
    }
    #[cfg(feature = "sonic")]
    if args.autocomplete {
        let search_term = &args
            .search_term
            .ok_or(anyhow::anyhow!("a search term is required"))?;
        let result = wiktionary_en_identifier_index::suggest(&language_to_use, search_term)?;
        utilities::pager::print_lines_in_pager(&result)?;
        return Ok(());
    }
    #[cfg(feature = "sonic")]
    if args.query {
        let search_term = &args
            .search_term
            .ok_or(anyhow::anyhow!("a search term is required"))?;
        let result = wiktionary_en_identifier_index::query(&language_to_use, search_term)?;
        utilities::pager::print_lines_in_pager(&result)?;
        return Ok(());
    }

    let mut result = run(
        QueryParameters {
            search_term: args.search_term,
            language: language_to_use,
            max_results: args.max_results,
            case_insensitive: args.case_insensitive,
            path: get_db_path(args.db_path, &language_to_use),
        },
        config_handler,
    )?;
    result.intercept()?;
    utilities::pager::print_in_pager(&result)
}

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use utilities::file_utils::*;
use utilities::language::*;

use wiktionary_en_entities::result::{DictionaryResult, DidYouMean, HistoryResult};

use wiktionary_en_db::client::{DbClient, DbClientMutex};

mod stats;
use stats::Stats;

mod result_wrapper;
use crate::result_wrapper::WiktionaryResultWrapper;

mod exhaustive_search;

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
    /// Search history
    #[clap(long)]
    history: bool,
}

struct QueryParameters {
    search_term: Option<String>,
    language: Language,
    max_results: usize,
    case_insensitive: bool,
    path: PathBuf,
}

#[cfg(feature = "sonic")]
fn search_for_alternative_term(
    client: &DbClient,
    query_params: &QueryParameters,
) -> Result<Option<DictionaryResult>> {
    if let Some(term) = &query_params.search_term {
        let did_you_mean =
            wiktionary_en_identifier_index::did_you_mean(&query_params.language, &term)?;
        if let Some(did_you_mean) = did_you_mean {
            let hits = client.find_by_word(&did_you_mean)?;
            if !hits.is_empty() {
                let result = DictionaryResult {
                    word: term.to_string(),
                    did_you_mean: Some(DidYouMean {
                        searched_for: term.to_string(),
                        suggestion: did_you_mean,
                    }),
                    hits,
                };
                return Ok(Some(result));
            }
        }
    }
    Ok(None)
}

fn search_for_term(
    client: &DbClient,
    term: &str,
    query_params: &QueryParameters,
) -> Result<DictionaryResult> {
    let hits = client.find_by_word(term)?;
    match hits.as_slice() {
        [_, ..] => Ok(DictionaryResult {
            word: term.to_string(),
            did_you_mean: None,
            hits,
        }),
        [] => {
            #[cfg(feature = "sonic")]
            if let Some(result) = search_for_alternative_term(&client, query_params)? {
                return Ok(result);
            }
            exhaustive_search::search(
                &query_params.path,
                term,
                query_params.max_results,
                query_params.case_insensitive,
            )
        }
    }
}

fn query_dictionary(
    client: &DbClient,
    query_params: QueryParameters,
    extension_handler: wiktionary_en_lua::ExtensionHandler,
) -> Result<WiktionaryResultWrapper> {
    if let Some(term) = &query_params.search_term {
        let result = search_for_term(client, term, &query_params)?;
        return Ok(WiktionaryResultWrapper {
            result: result_wrapper::WiktionaryResult::DictionaryResult(result),
            extension_handler,
        });
    }
    let hit = client.random_entry()?;
    let result = DictionaryResult {
        word: hit.word.clone(),
        did_you_mean: None,
        hits: vec![hit],
    };
    Ok(WiktionaryResultWrapper {
        result: result_wrapper::WiktionaryResult::DictionaryResult(result),
        extension_handler,
    })
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let config_handler = wiktionary_en_lua::ConfigHandler::init()?;
    let language_to_use = config_handler
        .config
        .parse_language_or_use_config_or_default(&args.language)?;

    if args.stats {
        let input_path = get_db_path(args.db_path, &language_to_use);
        let stats = Stats::calculate_stats(&input_path, &language_to_use)?;
        return utilities::pager::print_in_pager(&stats);
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
    let result = {
        let db_client = DbClient::init(language_to_use)?;
        let db_client_mutex = DbClientMutex::from(db_client.clone());
        let extension_handler = wiktionary_en_lua::ExtensionHandler::init(db_client_mutex)?;

        if args.history {
            let result = HistoryResult {
                history_entries: db_client.find_all_in_history()?,
            };
            let result_wrapper = WiktionaryResultWrapper {
                result: result_wrapper::WiktionaryResult::HistoryResult(result),
                extension_handler,
            };
            result_wrapper.fmt()?
        } else {
            let mut result = query_dictionary(
                &db_client,
                QueryParameters {
                    search_term: args.search_term,
                    language: language_to_use,
                    max_results: args.max_results,
                    case_insensitive: args.case_insensitive,
                    path: get_db_path(args.db_path, &language_to_use),
                },
                extension_handler,
            )?;

            result.intercept()?;
            result.fmt()?
        }
    };
    utilities::pager::print_in_pager(&result)
}

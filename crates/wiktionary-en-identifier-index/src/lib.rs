use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use utilities::file_utils;
use utilities::language::*;

use wiktionary_en_entities::wiktionary_entity::*;

use std::fs::File;

use sonic_channel::*;

use edit_distance::edit_distance;

const CANNOT_OPEN_SONIC_DB_ERROR_MSG: &str = "Couldn't open sonic db, please start it";

const WIKTIONARY_COLLECTION: &str = "wiktionary";

struct WiktionaryIngestChannel<'a> {
    language: &'a Language,
    ingest_channel: IngestChannel,
}

impl WiktionaryIngestChannel<'_> {
    pub fn init(language: &Language) -> Result<WiktionaryIngestChannel> {
        return Ok(WiktionaryIngestChannel {
            language: language,
            ingest_channel: start_sonic_ingest_channel()?,
        });
    }

    fn count(&self) -> Result<u64> {
        let number_of_objects = self.ingest_channel.count(CountRequest::objects(
            WIKTIONARY_COLLECTION,
            self.language.value(),
        ))?;
        return Ok(number_of_objects as u64);
    }

    pub fn statistics(&self) -> Result<()> {
        let bucket_count = self
            .ingest_channel
            .count(CountRequest::buckets(WIKTIONARY_COLLECTION))?;
        dbg!(bucket_count);

        let object_count = Self::count(self)?;
        dbg!(object_count);
        return Ok(());
    }

    pub fn flush(&self) -> Result<u64> {
        let flushdb_count = self.ingest_channel.flush(FlushRequest::bucket(
            WIKTIONARY_COLLECTION,
            self.language.value(),
        ))?;
        return Ok(flushdb_count as u64);
    }

    pub fn push(&self, word: &String) -> Result<()> {
        let obj = STANDARD.encode(word);
        let dest = Dest::col_buc(WIKTIONARY_COLLECTION, self.language.value()).obj(&obj);
        let push_result = self
            .ingest_channel
            .push(PushRequest::new(dest, word).lang(to_sonic_language(self.language)))?;
        return Ok(push_result);
    }
}

fn sonic_host() -> String {
    return env!("SONIC_HOST").to_string();
}

fn sonic_password() -> String {
    return env!("SONIC_PASSWORD").to_string();
}

fn start_sonic_ingest_channel() -> Result<IngestChannel> {
    let channel = IngestChannel::start(sonic_host(), sonic_password());
    return channel.map_err(|e| anyhow::Error::new(e).context(CANNOT_OPEN_SONIC_DB_ERROR_MSG));
}

fn start_sonic_search_channel() -> Result<SearchChannel> {
    let channel = SearchChannel::start(sonic_host(), sonic_password());
    return channel.map_err(|e| anyhow::Error::new(e).context(CANNOT_OPEN_SONIC_DB_ERROR_MSG));
}

fn check_line(line: Result<String, std::io::Error>, i: usize) -> Result<String> {
    return line.map_err(|e| {
        anyhow::Error::new(e).context(format!("Couldn't read line {} in DB file.", i))
    });
}

fn parse_line(line: &String, i: usize) -> Result<DictionaryEntry> {
    parse_entry(line).with_context(|| format!("Couldn't parse line {} in DB file.", i))
}

fn to_sonic_language(language: &Language) -> Lang {
    return match language {
        Language::EN => Lang::Eng,
        Language::DE => Lang::Deu,
        Language::SV => Lang::Swe,
        Language::FR => Lang::Fra,
        Language::ES => Lang::Spa,
    };
}

fn parse_and_persist(
    channel: &WiktionaryIngestChannel,
    file_reader: BufReader<File>,
) -> Result<()> {
    let flushb_count = channel.flush()?;
    dbg!(flushb_count);
    let mut count = 0;
    for (i, line) in file_reader.lines().enumerate() {
        let pushed = check_line(line, i).and_then(|line| {
            let dictionary_entry: DictionaryEntry = parse_line(&line, i)?;

            let push_result = channel.push(&dictionary_entry.word);
            if let Err(err) = push_result {
                println!(
                    "failed to index '{}' after {} iterations with error {}",
                    &dictionary_entry.word,
                    count,
                    err.to_string()
                );
            }
            count = i;
            return Ok(());
        });
        if let Err(e) = pushed {
            bail!(e);
        }
    }
    println!("iterated over {} entries", count);
    return Ok(());
}

pub fn statistics(language: &Language) -> Result<()> {
    let ingest_channel = WiktionaryIngestChannel::init(language)?;
    return ingest_channel.statistics();
}

pub fn suggest(language: &Language, search_term: &String) -> Result<Vec<String>> {
    // suggest queries for a term with spaces will restult in a server side error
    let first_word: String = search_term
        .chars()
        .take_while(|c| c != &' ' && c != &'-')
        .collect();
    let channel = start_sonic_search_channel()?;
    let suggestions = channel.suggest(SuggestRequest::new(
        Dest::col_buc("wiktionary", language.value()),
        &first_word,
    ))?;
    return Ok(suggestions);
}

pub fn query(language: &Language, search_term: &String) -> Result<Vec<String>> {
    let channel = start_sonic_search_channel()?;
    let objects = channel.query(
        QueryRequest::new(Dest::col_buc("wiktionary", language.value()), search_term)
            .lang(to_sonic_language(language)),
    )?;

    let mut terms: Vec<String> = Vec::new();
    for object in &objects {
        let decoded = STANDARD.decode(object)?;
        let term = String::from_utf8(decoded)?;
        terms.push(term);
    }
    return Ok(terms);
}

pub fn did_you_mean(language: &Language, search_term: &String) -> Result<Option<String>> {
    let mut alternatives = query(language, search_term)
        .context(format!("could't query for term '{}'", search_term))?;
    alternatives.append(
        &mut suggest(language, search_term)
            .context(format!("could't suggest for term '{}'", search_term))?,
    );
    let rated_suggestions = alternatives.iter().map(|suggestion| {
        let distance = edit_distance(search_term, suggestion);
        return (
            /* an exact match, that is distance 0, is not what we are looking for */
            if distance == 0 { usize::MAX } else { distance },
            suggestion,
        );
    });
    let best_result = rated_suggestions
        .min()
        .map(|rated_result| rated_result.1.to_string());

    return Ok(best_result);
}

pub fn generate_indices(language: &Language, db_path: &PathBuf, force: bool) -> Result<()> {
    let channel = WiktionaryIngestChannel::init(language)?;
    let number_of_objects = &channel.count()?;
    if *number_of_objects > 0 && !force {
        bail!(
            "{} indices already exists, use force to override",
            number_of_objects
        );
    }
    match file_utils::get_file_reader(db_path) {
        Ok(path) => return parse_and_persist(&channel, path),
        _ => bail!("No such DB file: '{}'", db_path.display()),
    }
}

use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use utilities::file_utils;
use utilities::language::*;

use wiktionary_en_entities::wiktionary_entity::*;

use std::fs::File;

use sonic_channel::*;

use edit_distance::edit_distance;

fn sonic_host() -> String {
    return env!("SONIC_HOST").to_string();
}

fn sonic_password() -> String {
    return env!("SONIC_PASSWORD").to_string();
}

fn start_sonic_ingest_channel() -> Result<IngestChannel> {
    let channel = IngestChannel::start(sonic_host(), sonic_password());
    return channel
        .map_err(|e| anyhow::Error::new(e).context("Couldn't open sonic db, please start it"));
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

fn parse_and_persist(file_reader: BufReader<File>, language: &Language) -> Result<()> {
    let channel = start_sonic_ingest_channel();

    let result = channel.and_then(|channel| {
        let flushb_count = channel.flush(FlushRequest::bucket("wiktionary", &language.value()))?;
        dbg!(flushb_count);
        let mut count = 0;
        for (i, line) in file_reader.lines().enumerate() {
            let pushed = check_line(line, i).and_then(|line| {
                let dictionary_entry: DictionaryEntry = parse_line(&line, i)?;
                let obj = STANDARD.encode(&dictionary_entry.word);
                let dest = Dest::col_buc("wiktionary", &language.value()).obj(&obj);

                let push_result = channel.push(
                    PushRequest::new(dest, &dictionary_entry.word)
                        .lang(to_sonic_language(language)),
                );
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
    });

    return result;
}

pub fn do_import(path: &Path, language: &Language) -> Result<()> {
    match file_utils::get_file_reader(path) {
        Ok(path) => return parse_and_persist(path, language),
        _ => bail!("No such DB file: '{}'", path.display()),
    }
}

pub fn statistics(language: &Language) -> Result<()> {
    let ingest_channel = start_sonic_ingest_channel()?;
    let bucket_count = ingest_channel.count(CountRequest::buckets("wiktionary"))?;
    dbg!(bucket_count);

    let object_count =
        ingest_channel.count(CountRequest::objects("wiktionary", language.value()))?;
    dbg!(object_count);
    return Ok(());
}

pub fn suggest(language: &Language, search_term: &String) -> Result<Vec<String>> {
    // suggest queries for a term with spaces will restult in a server side error
    let first_word: String = search_term.chars().take_while(|c| !c.eq(&' ')).collect();
    let channel = SearchChannel::start(sonic_host(), sonic_password())?;
    let suggestions = channel.suggest(SuggestRequest::new(
        Dest::col_buc("wiktionary", language.value()),
        &first_word,
    ))?;
    return Ok(suggestions);
}

pub fn query(language: &Language, search_term: &String) -> Result<Vec<String>> {
    let channel = SearchChannel::start(sonic_host(), sonic_password())?;
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
        (
            /* an exact match, that is distance 0, is not what we are looking for */
            std::cmp::max(1, edit_distance(search_term, suggestion)),
            suggestion,
        )
    });
    let best_result = rated_suggestions
        .min()
        .map(|rated_result| rated_result.1.to_string());

    return Ok(best_result);
}

pub fn generate_indices(language: &Language, db_path: &PathBuf, force: bool) -> Result<()> {
    if force {
        return do_import(db_path, language);
    }
    return Ok(());
}

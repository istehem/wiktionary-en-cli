use anyhow::{anyhow, bail, Context, Result};
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use utilities::file_utils;
use utilities::language::*;

use wiktionary_en_entities::wiktionary_entity::*;

use std::fs::File;

use sonic_channel::*;

fn start_sonic_ingest_channel() -> Result<IngestChannel> {
    let channel = IngestChannel::start("localhost:1491", "SecretPassword");
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
                let dest =
                    Dest::col_buc("wiktionary", &language.value()).obj(&dictionary_entry.word);

                let push_result = channel.push(
                    PushRequest::new(dest, &dictionary_entry.word)
                        .lang(to_sonic_language(language)),
                );
                if let Err(err) = push_result {
                    return Err(anyhow!(err).context(format!(
                        "failed for '{}' after {} iterations",
                        &dictionary_entry.word, count
                    )));
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

pub fn generate_indices(
    language: &Language,
    db_path: &PathBuf,
    search_term: &String,
    force: bool,
) -> Result<()> {
    println!("{}", utilities::DICTIONARY_DB_PATH!(Language::EN.value()));
    if force {
        return do_import(db_path, language);
    } else {
        let channel = SearchChannel::start("localhost:1491", "SecretPassword")?;

        let ingest_channel = start_sonic_ingest_channel()?;
        let bucket_count = ingest_channel.count(CountRequest::buckets("wiktionary"))?;
        dbg!(bucket_count);

        let object_count =
            ingest_channel.count(CountRequest::objects("wiktionary", language.value()))?;
        dbg!(object_count);

        let objects = channel.query(
            QueryRequest::new(Dest::col_buc("wiktionary", language.value()), search_term)
                .lang(to_sonic_language(language)),
        )?;
        dbg!(objects);
        let result = channel.suggest(SuggestRequest::new(
            Dest::col_buc("wiktionary", language.value()),
            search_term,
        ))?;
        dbg!(result);
    }
    return Ok(());
}

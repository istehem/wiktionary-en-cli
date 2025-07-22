use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader};

use wiktionary_en_entities::dictionary_entry::DictionaryEntry;

use std::fmt;
use std::fs::File;

use crate::channel::DictionaryIngestChannel;

use streaming_iterator::*;

pub struct IndexingError {
    iteration: usize,
    word: String,
    msg: String,
}

impl fmt::Display for IndexingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to index '{}' after {} iterations with error {}",
            &self.word, &self.iteration, &self.msg
        )
    }
}

type IndexingResponse = Result<Option<IndexingError>>;

pub struct IndexingStream {
    lines: std::io::Lines<BufReader<File>>,
    ingest_channel: DictionaryIngestChannel,
    current_line: Option<Result<String>>,
    indexing_response: IndexingResponse,
    index: usize,
    done: bool,
}

impl IndexingStream {
    pub fn from(file_reader: BufReader<File>, ingest_channel: DictionaryIngestChannel) -> Self {
        Self {
            lines: file_reader.lines(),
            ingest_channel,
            current_line: None,
            index: 0,
            done: false,
            indexing_response: Ok(None),
        }
    }
}

fn parse_and_push(
    channel: &DictionaryIngestChannel,
    line: &Result<String>,
    index: usize,
) -> Result<Option<IndexingError>> {
    match line {
        Ok(line) => {
            let dictionary_entry: DictionaryEntry = parse_line(line, index)?;
            let push_result = channel.push(&dictionary_entry.word);
            if let Err(err) = push_result {
                let indexing_error = IndexingError {
                    iteration: index,
                    word: dictionary_entry.word.clone(),
                    msg: err.to_string(),
                };
                return Ok(Some(indexing_error));
            }
            Ok(None)
        }
        Err(err) => bail!(err.to_string()),
    }
}

impl StreamingIterator for IndexingStream {
    type Item = IndexingResponse;

    fn advance(&mut self) {
        self.current_line = self.lines.next().map(|v| check_line(v, self.index));
        if let Some(line) = &self.current_line {
            self.indexing_response = parse_and_push(&self.ingest_channel, line, self.index);
        } else {
            self.done = true
        }
        self.index += 1;
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.done {
            return None;
        }
        Some(&self.indexing_response)
    }
}

fn check_line(line: Result<String, std::io::Error>, i: usize) -> Result<String> {
    line.map_err(|e| anyhow::Error::new(e).context(format!("Couldn't read line {} in DB file.", i)))
}

fn parse_line(line: &str, i: usize) -> Result<DictionaryEntry> {
    line.parse()
        .with_context(|| format!("Couldn't parse line {} in DB file.", i))
}

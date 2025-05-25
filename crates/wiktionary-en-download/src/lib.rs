use anyhow::{bail, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use futures_util::StreamExt;
use std::io::BufWriter;

use indicatif::{ProgressBar, ProgressStyle};
use utilities::language::*;

const PROGRESS_BAR_TEMPLATE: &str = "{wide_bar} {bytes}/{total_bytes}";

struct Writer {
    writer: BufWriter<File>,
    progress_bar: Option<ProgressBar>,
}

impl Writer {
    pub fn init(buf_writer: BufWriter<File>, content_length: Option<u64>) -> Result<Self> {
        let progress_bar = match content_length {
            Some(content_length) => Some(Self::init_progress_bar(content_length)?),
            _ => None,
        };
        let writer = Self {
            writer: buf_writer,
            progress_bar,
        };
        return Ok(writer);
    }

    fn init_progress_bar(content_length: u64) -> Result<ProgressBar> {
        let progress_bar = ProgressBar::new(content_length)
            .with_style(ProgressStyle::default_bar().template(PROGRESS_BAR_TEMPLATE)?);
        return Ok(progress_bar);
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write(&data)?;
        self.progress_bar
            .as_ref()
            .map(|progress_bar| progress_bar.inc(data.len() as u64));
        return Ok(());
    }

    pub fn flush(&mut self) -> Result<()> {
        let result = self.writer.flush().map_err(|err| anyhow::Error::new(err));
        self.progress_bar
            .as_ref()
            .map(|progress_bar| progress_bar.finish());
        return result;
    }
}

#[tokio::main]
async fn stream_download(url: &String, output_filename: &String) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client.get(url).send().await?;
    let content_length: Option<u64> = response.content_length();

    let mut bytes = response.bytes_stream();

    let file = File::create(output_filename)?;
    let buf_writer: BufWriter<File> = BufWriter::new(file);
    let mut writer = Writer::init(buf_writer, content_length)?;

    while let Some(chunk) = bytes.next().await {
        writer.write(&chunk?)?;
    }
    return writer.flush();
}

fn resource_url(language: &Language) -> String {
    let url = match language {
        Language::EN => "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl",
        Language::DE => "https://kaikki.org/dictionary/German/kaikki.org-dictionary-German.jsonl",
        Language::FR => "https://kaikki.org/dictionary/French/kaikki.org-dictionary-French.jsonl",
        Language::ES => "https://kaikki.org/dictionary/Spanish/kaikki.org-dictionary-Spanish.jsonl",
        Language::SV => "https://kaikki.org/dictionary/Swedish/kaikki.org-dictionary-Swedish.jsonl",
    };
    return String::from(url);
}

pub fn download_wiktionary_extract(language: &Language, force: bool) -> Result<()> {
    let url = resource_url(language);
    let output_filename = String::from(utilities::DICTIONARY_DB_PATH!(language.value()));
    if Path::new(&output_filename).exists() && !force {
        bail!(
            "file {} already exists, use force to override",
            &output_filename
        );
    }

    return stream_download(&url, &output_filename);
}

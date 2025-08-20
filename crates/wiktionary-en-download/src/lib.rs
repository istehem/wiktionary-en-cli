use anyhow::{bail, Result};
use std::path::Path;

use futures_util::StreamExt;

use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use utilities::language::Language;

const PROGRESS_BAR_TEMPLATE: &str = "{wide_bar} {bytes}/{total_bytes}";

struct Writer {
    writer: BufWriter<File>,
    progress_bar: Option<ProgressBar>,
}

impl Writer {
    fn init(buf_writer: BufWriter<File>, content_length: Option<u64>) -> Result<Self> {
        let progress_bar = match content_length {
            Some(content_length) => Some(Self::init_progress_bar(content_length)?),
            _ => None,
        };
        let writer = Self {
            writer: buf_writer,
            progress_bar,
        };
        Ok(writer)
    }

    fn init_progress_bar(content_length: u64) -> Result<ProgressBar> {
        let progress_bar = ProgressBar::new(content_length)
            .with_style(ProgressStyle::default_bar().template(PROGRESS_BAR_TEMPLATE)?);
        Ok(progress_bar)
    }

    async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data).await?;
        if let Some(progress_bar) = self.progress_bar.as_ref() {
            progress_bar.inc(data.len() as u64);
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        let result = self.writer.flush().await.map_err(anyhow::Error::new);
        if let Some(progress_bar) = self.progress_bar.as_ref() {
            progress_bar.finish();
        }
        result
    }
}

#[tokio::main]
async fn stream_download(url: &str, output_filename: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client.get(url).send().await?;
    let content_length: Option<u64> = response.content_length();

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(10); // Buffer size of 10

    let reader_task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
        let mut bytes = response.bytes_stream();
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk?;
            if tx.send(chunk.to_vec()).await.is_err() {
                break;
            }
        }
        Ok(())
    });

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_filename)
        .await?;

    let buf_writer: BufWriter<File> = BufWriter::new(file);
    let mut writer = Writer::init(buf_writer, content_length)?;

    while let Some(chunk) = rx.recv().await {
        writer.write_all(&chunk).await?;
    }
    writer.flush().await?;

    reader_task.await??;

    Ok(())
}

fn resource_url(language: &Language) -> String {
    let url = match language {
        Language::EN => "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl",
        Language::DE => "https://kaikki.org/dictionary/German/kaikki.org-dictionary-German.jsonl",
        Language::FR => "https://kaikki.org/dictionary/French/kaikki.org-dictionary-French.jsonl",
        Language::ES => "https://kaikki.org/dictionary/Spanish/kaikki.org-dictionary-Spanish.jsonl",
        Language::SV => "https://kaikki.org/dictionary/Swedish/kaikki.org-dictionary-Swedish.jsonl",
    };
    String::from(url)
}

fn download_dictionary_extract(language: &Language, force: bool) -> Result<()> {
    let url = resource_url(language);
    let output_filename = utilities::DICTIONARY_DB_PATH!(language);
    if Path::new(&output_filename).exists() && !force {
        bail!(
            "file {} already exists, use force to override",
            &output_filename
        );
    }

    stream_download(&url, &output_filename)
}

pub struct Downloader {}

impl Downloader {
    pub fn download_dictionary_extract(language: &Language, force: bool) -> Result<()> {
        download_dictionary_extract(language, force)
    }
}

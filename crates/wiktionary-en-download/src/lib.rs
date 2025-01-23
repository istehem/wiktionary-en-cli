use anyhow::{bail, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use futures_util::StreamExt;
use std::io::BufWriter;

use indicatif::{ProgressBar, ProgressStyle};
use utilities::language::*;

#[tokio::main]
async fn stream_download(url: &String, output_filename: &String) -> Result<()> {
    let client = reqwest::Client::new();

    let response = client.get(url).send().await?;
    let content_length: Option<u64> = response.content_length();
    let progress_bar = ProgressBar::new(content_length.unwrap())
        .with_style(ProgressStyle::default_bar().template("{wide_bar} {bytes}/{total_bytes}")?);

    let mut bytes = response.bytes_stream();

    let file = File::create(output_filename)?;
    let mut writer: BufWriter<File> = BufWriter::new(file);

    while let Some(chunk) = bytes.next().await {
        let data = chunk?;
        writer.write(&data)?;
        progress_bar.inc(data.len() as u64);
    }
    progress_bar.finish();
    return writer.flush().map_err(|err| anyhow::Error::new(err));
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

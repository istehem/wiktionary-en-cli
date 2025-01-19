use anyhow::Result;
use std::fs::File;
use std::io::Write;

use futures_util::StreamExt;
use std::io::BufWriter;

use utilities::language::*;

#[tokio::main]
async fn stream_download(url: &String, output_filename: &String) -> Result<()> {
    let client = reqwest::Client::new();

    let mut bytes = client.get(url).send().await?.bytes_stream();

    let file = File::create(output_filename)?;
    let mut writer: BufWriter<File> = BufWriter::new(file);

    while let Some(item) = bytes.next().await {
        writer.write(&item?)?;
    }
    return writer.flush().map_err(|err| anyhow::Error::new(err));
}

fn resource_url(language: Language) -> String {
    let url = match language {
        Language::EN => "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl",
        Language::DE => "https://kaikki.org/dictionary/German/kaikki.org-dictionary-German.jsonl",
        Language::FR => "https://kaikki.org/dictionary/French/kaikki.org-dictionary-French.jsonl",
        Language::ES => "https://kaikki.org/dictionary/Spanish/kaikki.org-dictionary-Spanish.jsonl",
        Language::SV => "https://kaikki.org/dictionary/Swedish/kaikki.org-dictionary-Swedish.jsonl",
    };
    return String::from(url);
}

pub fn download() -> Result<()> {
    let url = resource_url(Language::EN);
    let output_filename = String::from("wiktionary-en.jsonl");
    return stream_download(&url, &output_filename);
}

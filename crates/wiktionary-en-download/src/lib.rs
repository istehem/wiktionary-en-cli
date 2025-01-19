use anyhow::Result;
use std::fs::File;
use std::io::Write;

use futures_util::StreamExt;
use std::io::BufWriter;

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

pub fn download() -> Result<()> {
    //let url = "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl";
    //let url = "https://kaikki.org/dictionary/German/kaikki.org-dictionary-German.jsonl";
    //let url = "https://kaikki.org/dictionary/Swedish/kaikki.org-dictionary-Swedish.jsonl";
    //let url = String::from("https://kaikki.org/dictionary/French/kaikki.org-dictionary-French.jsonl");
    let url =
        String::from("https://kaikki.org/dictionary/Spanish/kaikki.org-dictionary-Spanish.jsonl");
    let output_filename = String::from("wiktionary-es.jsonl");
    return stream_download(&url, &output_filename);
}

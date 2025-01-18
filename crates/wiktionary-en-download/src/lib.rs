use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

use futures_util::StreamExt;
use std::io::BufWriter;

#[tokio::main]
async fn stream_download() -> Result<()> {
    let client = reqwest::Client::new();

    let url = "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl";
    //let url = "https://www.pixelstalk.net/wp-content/uploads/2016/08/Cute-Girl-HD-Images.jpg";
    let mut bytes = client
        .get(url)
        .timeout(Duration::from_secs(1800))
        .send()
        .await
        .unwrap()
        .bytes_stream();

    let file = File::create("test.jsonl")?;
    let mut writer: BufWriter<File> = BufWriter::new(file);

    while let Some(item) = bytes.next().await {
        writer.write(&item?)?;
    }

    return Ok(());
}

pub fn download() -> Result<()> {
    return stream_download();
}

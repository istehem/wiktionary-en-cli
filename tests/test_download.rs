#[cfg(feature = "test-download")]
mod tests {
    use anyhow::Result;
    use rstest::*;
    use std::time::Instant;
    use wiktionary_en_download::Downloader;

    #[rstest]
    fn test_download() -> Result<()> {
        let url = "https://testfileorg.netwet.net/500MB-CZIPtestfile.org.zip";
        let file_name = "./tmp/download_test.zip";

        let start = Instant::now();
        Downloader::download(url, file_name)?;
        let duration = start.elapsed();
        println!("Download of {} took: {:?}", url, duration);
        Ok(())
    }
}

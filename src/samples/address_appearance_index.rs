use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use reqwest::Url;
use tokio::{fs::File, io::AsyncWriteExt, runtime::Runtime};

use super::traits::SampleObtainer;

pub struct AAISampleObtainer;

impl SampleObtainer for AAISampleObtainer {
    fn raw_sample_filenames() -> Vec<&'static str> {
        return SAMPLE_CHUNKS.to_vec();
    }
    fn processed_sample_filenames() -> Vec<&'static str> {
        todo!()
    }

    fn get_raw_samples(dir: &PathBuf) -> Result<()> {
        let rt = Runtime::new()?;
        rt.block_on(download_unchained_samples(dir))
    }
}

/// Downloads the sample Unchained Index chunk files from IPFS.
///
/// Saves five 25MB files locally in the sample directory.
async fn download_unchained_samples(chunks_dir: &PathBuf) -> Result<()> {
    // Download from lib repo.
    let client = reqwest::Client::new();
    fs::create_dir_all(&chunks_dir)?;
    for (index, chunk_name) in SAMPLE_CHUNK_CIDS.iter().enumerate() {
        let url = Url::parse(SAMPLE_UNCHAINED_DIR)?.join(chunk_name)?;
        let filename = chunks_dir.join(SAMPLE_CHUNKS[index]);
        println!("Downloading chunk by CID: {}", url);
        let mut file = File::create(filename).await?;
        let mut stream = client.get(url).send().await?.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
    }
    println!(
        "Downloaded five Unchained Index sample files to: {:?}",
        &chunks_dir
    );
    Ok(())
}

static SAMPLE_CHUNKS: [&str; 5] = [
    "011283653-011286904.bin",
    "012387154-012389462.bin",
    "013408292-013411054.bin",
    "014482581-014485294.bin",
    "015508866-015511829.bin",
];

pub static SAMPLE_CHUNK_CIDS: [&str; 5] = [
    "QmNpXdysAvS9PzEjnG6WeX18G9pxAa1mwL6TePrttV7XUM",
    "QmanGdgER53dayvG61zudQewdRSpx93ELWxxui9QiJRqwr",
    "QmQyxxDddM6khNKjPnnj6LxXC6hjjrtY4BdyVCSmHhfMwn",
    "QmaiaJEhHvzgizhJZvn2MoEx13M1MRBKHKz7MDeCELyXVx",
    "Qmegr6DCEQ6Si1FZbbRZJFhXWM9hWbG7PnYcEGFGkPuJuB",
];

static SAMPLE_UNCHAINED_DIR: &str = "https://ipfs.unchainedindex.io/ipfs/";

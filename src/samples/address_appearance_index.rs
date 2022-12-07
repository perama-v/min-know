use std::path::PathBuf;

use anyhow::Result;

use reqwest::Url;
use tokio::runtime::Runtime;

use super::traits::SampleObtainer;
use super::utils::download_files;

pub struct AAISampleObtainer;

impl SampleObtainer for AAISampleObtainer {
    fn raw_sample_filenames() -> Vec<&'static str> {
        return SAMPLE_CHUNKS.to_vec();
    }
    fn processed_sample_filenames() -> Option<Vec<&'static str>> {
        None
    }

    /// Downloads the sample Unchained Index chunk files from IPFS.
    ///
    /// Saves five 25MB files locally in the sample directory.
    fn get_raw_samples(dir: &PathBuf) -> Result<()> {
        let mut urls_and_filenames: Vec<(Url, &str)> = vec![];
        for (index, chunk_name) in SAMPLE_CHUNK_CIDS.iter().enumerate() {
            urls_and_filenames.push((
                Url::parse(SAMPLE_UNCHAINED_DIR)?.join(chunk_name)?,
                SAMPLE_CHUNKS[index],
            ))
        }
        println!(
            "Downloaded {} files to: {:?}",
            urls_and_filenames.len(),
            dir
        );
        let rt = Runtime::new()?;
        rt.block_on(download_files(&dir, urls_and_filenames))
    }
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

use std::path::Path;

use anyhow::Result;
use log::info;
use reqwest::Url;
use tokio::runtime::Runtime;

use crate::samples::traits::SampleObtainerMethods;
use crate::utils::download::{download_files, DownloadTask};

pub struct AAISampleObtainer;

impl SampleObtainerMethods for AAISampleObtainer {
    fn raw_sample_filenames() -> Vec<&'static str> {
        SAMPLE_CHUNKS.to_vec()
    }
    fn sample_volumes() -> Option<Vec<&'static str>> {
        Some(SAMPLE_VOLUMES.to_vec())
    }

    /// Downloads the sample Unchained Index chunk files from IPFS.
    ///
    /// Saves five 25MB files locally in the sample directory.
    fn get_raw_samples(dir: &Path) -> Result<()> {
        let mut tasks: Vec<DownloadTask> = vec![];
        for (index, chunk_name) in SAMPLE_CHUNK_CIDS.iter().enumerate() {
            tasks.push(DownloadTask {
                url: Url::parse(SAMPLE_UNCHAINED_URL)?.join(chunk_name)?,
                dest_dir: dir.to_path_buf(),
                filename: SAMPLE_CHUNKS[index].to_string(),
            })
        }
        info!("Downloading {} files to: {:?}", tasks.len(), dir);
        let rt = Runtime::new()?;
        rt.block_on(download_files(tasks))?;

        Ok(())
    }
}

static SAMPLE_VOLUMES: [&str; 4] = [
    "volume_011_200_000",
    "volume_012_300_000",
    "volume_013_400_000",
    "volume_014_400_000",
];

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

static SAMPLE_UNCHAINED_URL: &str = "https://ipfs.unchainedindex.io/ipfs/";

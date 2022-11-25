//! This module can be used to obtain index-related data of different kinds.
use anyhow::{anyhow, Result};
use futures_util::stream::StreamExt;
use reqwest::Url;
use std::{fs, path::PathBuf};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    constants::NUM_CHAPTERS,
    transform::full_transform,
    types::{self, AddressIndexPath, Network, UnchainedPath},
    utils::{chapter_dir_name, volume_file_name},
};

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

static SAMPLE_VOLUMES: [u32; 4] = [11_200_000, 12_300_000, 13_400_000, 14_400_000];

/// Fetches the sample data and places it in the project data directory.
///
/// If the sample data is already present in the local directory, it copies it to
/// a standard location, determined by the operating system (see AddressIndexPath).
///
/// If the sample data is absent, it downloads some Unchained Index chunk files and
/// parses them to create a sample of the index locally.
pub async fn samples(
    index_path: &AddressIndexPath,
    unchained_path: &UnchainedPath,
    network: &Network,
) -> Result<()> {
    // Need both the address-appearance-index and the Unchained Index samples.
    get_unchained_samples(unchained_path, network).await?;
    get_address_appearance_index_samples(unchained_path, index_path, network)?;
    Ok(())
}

/// Obtains the Unchained Index sample files.
async fn get_unchained_samples(path: &UnchainedPath, network: &Network) -> Result<()> {
    // Are the samples in the right place?
    match unchained_samples_present(&path, network) {
        Ok(true) => {
            println!(
                "The Unchained sample files are already present in {:?}",
                path.chunks_dir(network)?
            );
            return Ok(());
        }
        _ => {}
    }
    // Try local directory. Then copy to samples dir.
    let local_sample_dir = UnchainedPath::Custom(PathBuf::from("./data"));
    match unchained_samples_present(&local_sample_dir, network) {
        Ok(true) => {
            // Need to copy samples from local to desired directory.
            return move_unchained_samples(&local_sample_dir, path, network);
        }
        _ => {}
    }
    download_unchained_samples(path, network).await?;
    Ok(())
}

/// Downloads the sample Unchained Index chunk files from IPFS.
///
/// Saves five 25MB files locally in the sample directory.
async fn download_unchained_samples(path: &UnchainedPath, network: &Network) -> Result<()> {
    // Download from lib repo.
    let client = reqwest::Client::new();
    let chunks_dir = path.chunks_dir(&network)?;
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

/// Obtains the address-appearance-index sample files
/// by deriving them.
fn get_address_appearance_index_samples(
    unchained_path: &UnchainedPath,
    desired_path: &AddressIndexPath,
    network: &Network,
) -> Result<()> {
    // Check if appearance index files are in right location.
    match appearance_index_samples_present(&desired_path, network) {
        Ok(true) => {
            println!(
                "The address-appearance-idex sample files are already present in {:?}",
                desired_path.index_dir(network)?
            );
            return Ok(());
        }
        _ => {}
    }
    // Check if appearance index are in local directory.
    let local_sample_dir = types::AddressIndexPath::Custom(PathBuf::from("./data/samples"));
    match appearance_index_samples_present(&local_sample_dir, network) {
        Ok(true) => {
            // Need to move from local dir to desired dir.
            //todo!("prevent copying if files already present");
            move_address_appearance_index_samples(&local_sample_dir, desired_path, network)?;
            return Ok(());
        }
        _ => {}
    }
    // The samples can be derived from the unchained samples.
    println!("Did not find the address-appearance-index sample files. Deriving them from the Unchained Index sample files.");
    full_transform(unchained_path, desired_path, network)?;
    Ok(())
}

/// Tests if a few expected files are present and returns an error if not.
///
/// # Errors
/// Returns an error if the manifest and the volume file cannot be read.
pub fn appearance_index_samples_present(
    path: &AddressIndexPath,
    network: &Network,
) -> Result<bool> {
    let the_manifest = path.manifest_file(network)?;
    fs::read(the_manifest.as_path())?;
    let a_volume = path
        .chapter_dir(network, "5a")?
        .join(volume_file_name("5a", 11_200_000)?);
    fs::read(a_volume.as_path())?;
    Ok(true)
}

/// Tests if the Unchained Index sample chunk files are present.
pub fn unchained_samples_present(path: &UnchainedPath, network: &Network) -> Result<bool> {
    let file_path = path.chunks_dir(network)?.join(SAMPLE_CHUNKS[0]);
    fs::read(&file_path)?;
    Ok(true)
}

/// Moves local Unchained sample files to desired directory.
///
/// Copies ./data/samples/unchained_index_NETWORK to xyz/samples/unchained_index_NETWORK
fn move_unchained_samples(
    local: &UnchainedPath,
    desired: &UnchainedPath,
    network: &Network,
) -> Result<()> {
    let local_chunks_path = local.chunks_dir(network)?;
    let desired_chunks_path = desired.chunks_dir(network)?;
    for chunk in SAMPLE_CHUNKS {
        fs::create_dir_all(&desired_chunks_path)?;
        fs::copy(
            local_chunks_path.join(chunk),
            desired_chunks_path.join(chunk),
        )?;
    }
    println!(
        "Copied sample dir \n\tfrom:{:?} \n\tto: {:?}.",
        local_chunks_path, desired_chunks_path
    );
    Ok(())
}

/// Moves local address-appearance-index sample files to desired directory.
///
/// Copies ./data/samples/address_appearance_index_NETWORK to
/// xyz/samples/address_appearance_index_NETWORK
fn move_address_appearance_index_samples(
    local: &AddressIndexPath,
    desired: &AddressIndexPath,
    network: &Network,
) -> Result<()> {
    let local_index_path = local.index_dir(network)?;
    let desired_index_path = desired.index_dir(network)?;

    for chap_num in 0..NUM_CHAPTERS {
        let chap_hex = format!("{:0>2x}", chap_num);
        let chap_name = chapter_dir_name(&chap_hex);
        for num in SAMPLE_VOLUMES {
            let filename = volume_file_name(&chap_hex, num)?;
            fs::create_dir_all(&desired_index_path.join(&chap_name))?;
            let from = local_index_path.join(&chap_name).join(&filename);
            let to = desired_index_path.join(&chap_name).join(&filename);
            fs::copy(from, to)?;
        }
    }
    let manifest = local.manifest_file(network)?;
    let manifest_name = manifest
        .file_name()
        .ok_or_else(|| anyhow!("No manifest file name."))?;
    let to = desired_index_path.join(manifest_name);
    fs::copy(manifest, to)?;

    println!(
        "Copied sample dir including manifest \n\tfrom:{:?} \n\tto: {:?}.",
        local_index_path, desired_index_path
    );
    Ok(())
}

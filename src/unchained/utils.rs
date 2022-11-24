use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::{fs, path::PathBuf};

use super::{
    constants::{AD_ENTRY, AP_ENTRY, HEAD},
    structure::{Body, Header, Section, TransactionId},
    types::{BlockRange, UnchainedFile},
};

/// Details for files in the Unchained Index chunk directory.
pub struct ChunksDir {
    pub dir: PathBuf,
    pub paths: Vec<ChunkFile>,
}

impl ChunksDir {
    /// Obtains information about all the available chunk files.
    ///
    /// # Example
    /// If the chunk files are in "xyz/trueblocks/unchained/mainnet/finalized",
    /// then this is the path passed in.
    pub fn new(dir_path: &PathBuf) -> Result<Self> {
        let files = fs::read_dir(dir_path)
            .with_context(|| format!("Failed to read dir from {:?}", dir_path))?;
        let mut paths: Vec<ChunkFile> = vec![];
        for file in files {
            let path = file?.path();
            let range = get_range(&path)?;
            let chunk = ChunkFile { path, range };
            paths.push(chunk);
        }

        paths.sort_by_key(|k| k.range.old);
        Ok(ChunksDir {
            dir: dir_path.to_path_buf(),
            paths,
        })
    }
    /// Obtains the details of chunk files relevant for a given block range.
    ///
    /// Chunks are relevant if they intersect the desired range.
    pub fn for_range(&self, desired_range: &BlockRange) -> Result<Vec<&ChunkFile>> {
        let mut relevant: Vec<&ChunkFile> = vec![];
        for chunk in &self.paths {
            if chunk.range.intersection_exists(desired_range) {
                relevant.push(chunk);
            }
        }
        Ok(relevant)
    }
}

#[derive(Clone, Debug)]
pub struct ChunkFile {
    pub path: PathBuf,
    pub range: BlockRange,
}

/// Determines the byte indices for a given chunk file.
pub fn file_structure(h: &Header) -> Body {
    let app_start = HEAD + h.n_addresses as usize * AD_ENTRY;
    let total_bytes = app_start + h.n_appearances as usize * AP_ENTRY;
    Body {
        addresses: Section {
            start: HEAD,
            current: HEAD,
            end: app_start - 1,
        },
        appearances: Section {
            start: app_start,
            current: app_start,
            end: total_bytes - 1,
        },
    }
}

/// Get first and last block that an index chunk covers.
pub fn get_range(path: &PathBuf) -> anyhow::Result<BlockRange> {
    // Two 9 digit values .../123456789-123456789.bin
    let path_string = path
        .to_str()
        .ok_or_else(|| anyhow!("Cannot read path {:?} as string.", path))?;
    let bounds = Regex::new(
        r"(?x)
    (?P<low>\d{9})  # the earliest block.
    -
    (?P<high>\d{9}) # the the latest block.
    ",
    )?
    .captures(path_string)
    .ok_or_else(|| anyhow!("file {} title lacks 9-digit block range", path_string))?;
    Ok(BlockRange {
        old: bounds["low"].parse::<u32>()?,
        new: bounds["high"].parse::<u32>()?,
    })
}

/// Checks that given appearance is within chunk file bounds.
pub fn no_unexpected_appearances(
    appearance: &TransactionId,
    uf: &UnchainedFile,
) -> anyhow::Result<()> {
    if appearance.block < uf.present.old || appearance.block > uf.present.new {
        return Err(anyhow!(
            "file {:?} has appearance out of expected range ({}-{}). {:?}",
            uf.path,
            uf.present.old,
            uf.present.new,
            appearance
        ));
    }
    Ok(())
}

use std::{
    collections::{hash_map::Entry, HashMap},
    path::Path,
};

use anyhow::{anyhow, bail, Result};

use crate::{
    parameters::address_appearance_index::BLOCKS_PER_VOLUME,
    specs::address_appearance_index::{
        AAIAppearanceTx, AAIChapter, AAIChapterId, AAISpec, AAIVolumeId, RelicAddressAppearances,
        RelicChapter, RelicVolumeIdentifier,
    },
    utils::unchained::{
        files::{ChunkFile, ChunksDir},
        structure::TransactionId,
        types::{BlockRange, UnchainedFile},
    },
};

use super::traits::ExtractorMethods;

pub struct AAIExtractor {}

impl ExtractorMethods<AAISpec> for AAIExtractor {
    fn chapter_from_raw(
        chapter_id: &AAIChapterId,
        volume_id: &AAIVolumeId,
        source_dir: &Path,
    ) -> Result<Option<AAIChapter>> {
        // Get relevant raw files.
        let chunk_files: ChunksDir = ChunksDir::new(source_dir)?;
        let block_range = volume_id.to_block_range()?;
        let Some(relevant_files) = chunk_files.for_range(&block_range) else {
            return Ok(None)
        };
        // Get appearances from files.
        let leading_char = hex::encode(chapter_id.val.to_vec());
        // This (RelicChapter->AAIChapter) is a workaround to use existing code.
        // Ideally get_relevant_appearances() returns AAIChapter directly.
        let relic_chapter: RelicChapter =
            get_relevant_appearances(relevant_files, block_range, &leading_char)?;
        let chapter = AAIChapter::from_relic(relic_chapter);
        Ok(Some(chapter))
    }
    fn latest_possible_volume(source_dir: &Path) -> Result<AAIVolumeId> {
        let chunk_files: ChunksDir = ChunksDir::new(source_dir)?;
        Ok(AAIVolumeId {
            oldest_block: latest_full_volume(latest_block_in_chunks(&chunk_files)?)?,
        })
    }
}

/// For the given Unchained Index chunk files, finds transactions that match
/// The desired block range and address leading hex characters.
pub fn get_relevant_appearances(
    chunk_file_paths: Vec<&ChunkFile>,
    desired: BlockRange,
    leading_char: &str,
) -> Result<RelicChapter> {
    let mut relevant_appearances: HashMap<Vec<u8>, Vec<TransactionId>> = HashMap::new();
    for chunk in chunk_file_paths {
        let path = chunk.path.to_owned();
        // File reader
        let mut uf: UnchainedFile = UnchainedFile::new(path, desired)?;
        // Read appearances that have correct leading char and are in desired range.
        uf.with_parsed(leading_char)?;
        // Add or update as appropriate.
        for to_add in uf.parsed {
            let key = to_add.address;
            match relevant_appearances.entry(key) {
                Entry::Occupied(mut entry) => {
                    // Append to existing array and insert.
                    entry.get_mut().extend(to_add.appearances);
                }
                Entry::Vacant(entry) => {
                    // Insert.
                    entry.insert(to_add.appearances);
                }
            }
        }
    }
    // Convert from hashmap to vector.
    let mut addresses: Vec<RelicAddressAppearances> = relevant_appearances
        .into_iter()
        .map(|(key, val)| RelicAddressAppearances {
            address: <_>::from(key),
            appearances: {
                // Start with Vec<TransactionId>
                // Get Vec<AppearanceTx>
                let t: Vec<AAIAppearanceTx> = val
                    .iter()
                    .map(|x| AAIAppearanceTx {
                        block: x.block,
                        index: x.index,
                    })
                    .collect();
                // Get AppearanceTxList
                <_>::from(t)
            },
        })
        .collect();
    // Sort lexicographically by address. E.g., [0x0a, 0xa0, 0xaa].
    addresses.sort_by(|a, b| a.address.cmp(&b.address));

    let address_as_hex = hex::decode(leading_char)?;
    let res = RelicChapter {
        address_prefix: <_>::from(address_as_hex),
        identifier: RelicVolumeIdentifier {
            oldest_block: desired.old,
        },
        addresses: <_>::from(addresses),
    };
    Ok(res)
}

/// Finds the latest block in an Unchained Index chunks directory.
///
/// If the chunks directory contains the latest chunk: "015433333-015455555.bin"
/// the value 15_455_555 will be returned.
pub fn latest_block_in_chunks(chunks: &ChunksDir) -> Result<u32> {
    let latest = chunks
        .paths
        .last()
        .ok_or_else(|| {
            anyhow!(
                "Expected chunks dir {:?} to contain files, found none.",
                chunks.dir
            )
        })?
        .range
        .new;
    Ok(latest)
}

/// Gets the latest complete volume possible for a given block height (as
/// a VolumeId)
///
/// latest block, id of latest full volume:
/// - 99_999, 0
/// - 199_999, 100_000
/// - 200_000, 100_000
/// - 299_998, 100_000
/// - 299_999, 200_000
pub fn latest_full_volume(highest_block: u32) -> Result<u32> {
    if highest_block < BLOCKS_PER_VOLUME - 1 {
        bail!("No complete blocks possible")
    }

    Ok(((highest_block + 1 - BLOCKS_PER_VOLUME) / BLOCKS_PER_VOLUME) * BLOCKS_PER_VOLUME)
}

#[test]
fn test_latest_vol_id() {
    assert!(matches!(latest_full_volume(99_998), Err(_error)));
    assert_eq!(latest_full_volume(99_999).unwrap(), 0);
    assert_eq!(latest_full_volume(199_999).unwrap(), 100_000);
    assert_eq!(latest_full_volume(200_000).unwrap(), 100_000);
    assert_eq!(latest_full_volume(299_998).unwrap(), 100_000);
    assert_eq!(latest_full_volume(299_999).unwrap(), 200_000);
}

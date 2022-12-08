use std::{
    collections::{hash_map::Entry, HashMap},
    path::PathBuf,
};

use anyhow::Result;

use crate::{
    specs::{
        address_appearance_index::{
            AAIAppearanceTx, AAIChapter, AAISpec, RelicAddressAppearances, RelicChapter,
            RelicVolumeIdentifier,
        },
        traits::{ChapterIdMethods, DataSpec},
    },
    unchained::{
        structure::TransactionId,
        types::{BlockRange, UnchainedFile},
        utils::{ChunkFile, ChunksDir},
    },
};

use super::traits::Extractor;

pub struct AAIExtractor {}

impl Extractor<AAISpec> for AAIExtractor {
    fn chapter_from_raw(
        chapter_id: &<AAISpec as DataSpec>::AssociatedChapterId,
        volume_id: &<AAISpec as DataSpec>::AssociatedVolumeId,
        source_dir: &PathBuf,
    ) -> Result<<AAISpec as DataSpec>::AssociatedChapter> {
        // Get relevant raw files.
        let chunk_files: ChunksDir = ChunksDir::new(&source_dir)?;
        let block_range = volume_id.to_block_range()?;
        let relevant_files: Vec<&ChunkFile> = chunk_files.for_range(&block_range)?;
        // Get appearances from files.
        let relic_chapter: RelicChapter =
            get_relevant_appearances(relevant_files, block_range, &chapter_id.interface_id())?;
        let chapter = AAIChapter::from_relic(relic_chapter);
        Ok(chapter)
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

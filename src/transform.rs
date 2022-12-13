//! For creating a derivative index using the Unchained Index.
//!
//! Used to transform the Unchained Index into the address-appearance-index.
use anyhow::{anyhow, Context, Result};
use std::collections::{hash_map::Entry, HashMap};
use std::fs;

use crate::constants::NUM_CHAPTERS;
use crate::{
    constants::BLOCKS_PER_VOLUME,
    encoding,
    spec::{AddressAppearances, AddressIndexVolumeChapter, AppearanceTx, VolumeIdentifier},
    types::{AddressIndexPath, Network, UnchainedPath},
    unchained::{
        structure::TransactionId,
        types::{BlockRange, UnchainedFile},
        utils::{ChunkFile, ChunksDir},
    },
    utils::{self, volume_id_to_block_range},
};
/// Creates the full address-appearance-index using the Unchained Index.
///
/// Creates a file system with one folder per [address chapter][1], each with [volume][2]
/// serialized and compressed [volume][2] files. The file names are labelled
/// according to the [string naming conventions][3].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#addresschapter
/// [2]: https://github.com/perama-v/address-appearance-index-specs#addressindexvolumechapter
/// [3]: https://github.com/perama-v/address-appearance-index-specs#string-naming-conventions
///
/// # Example
/// See ./examples/maintainer_create_index.rs for more.
/// ```no_run
/// use min_know::{
///     transform::full_transform,
///     types::{AddressIndexPath, Network, UnchainedPath}};
///
/// let from = UnchainedPath::Default;
/// let to = AddressIndexPath::Default;
/// let network = Network::default();
/// full_transform(&from, &to, &network)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// The extraction excludes all chunk files that are part of an incomplete
/// volume. That is, if the final chunk in the Unchained Index is
/// `015531844-015534579.bin`, then the highest included block
/// in the address-appearance-index will be `15_499_999`
/// (assuming a volume RANGE is 100_000). Blocks `15_500_000` to `15_599_999`
/// will be included when the chunks include block `15_600_000`.
///
/// See algorithm in the [spec][1] under "Procedures -> Maintenance: create index"
/// Algorithm:
/// - For each address chapter (0x00, 0x01, ... 0xff) (x256) create new directory.
///     - For each volume (100_000 block range) (x~150-200) define a new file.
///         - Get all unchained index chunk files relevant for that range (x~10-20).
///             - For each chunk file find transactions that match the address volume and
///             chapter.
///         - Hold these transactions in memory (x~100_000-300_000 txs) in a struct
///         and ssz serialize when done.
///         - Save the ssz_root_hash in a manifest
///         - Perform snappy compression
///     - Save compressed data under that key.
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs
pub fn full_transform(
    source: &UnchainedPath,
    destination: &AddressIndexPath,
    network: &Network,
) -> Result<()> {
    let chunks_path = source.chunks_dir(network)?;
    let chunk_files: ChunksDir = ChunksDir::new(&chunks_path)?;
    let latest_mainnet = latest_block_in_chunks(&chunk_files)?;
    let chapter_dirs = get_chapter_volumes(0, latest_mainnet)?;

    create_specific_volume_files(destination, network, chapter_dirs, chunk_files)?;
    Ok(())
}

fn create_specific_volume_files(
    destination: &AddressIndexPath,
    network: &Network,
    chapter_dirs: Vec<ChapterDirectoryIdentifier>,
    chunk_files: ChunksDir,
) -> Result<bool> {
    let mut modified_index = false;
    let destination_path = destination.index_dir(network)?;
    for chapter_info in chapter_dirs {
        // One directory for each address chapter.
        let chap_name = utils::chapter_dir_name(&chapter_info.leading_chars);
        let chap_path = destination_path.join(chap_name);
        fs::create_dir_all(&chap_path)?;

        for volume_info in chapter_info.volumes {
            // One file for each range-defined volume.
            let relevant_files = match chunk_files.for_range(&volume_info) {
                Some(files) => files,
                None => vec![],
            };
            let volume: AddressIndexVolumeChapter =
                get_relevant_appearances(relevant_files, volume_info, &chapter_info.leading_chars)?;

            let txs_total = volume
                .addresses
                .iter()
                .fold(0, |acc, x| acc + x.appearances.len());
            if txs_total != 0 {
                let file_name = utils::volume_file_name(
                    &chapter_info.leading_chars,
                    volume.identifier.oldest_block,
                )?;
                println!(
                    "Creating file: {} will have {} addresses and {} transactions.",
                    &file_name,
                    volume.addresses.len(),
                    txs_total
                );
                let ssz_snappy = encoding::encode_and_compress(volume)?;
                let filepath = chap_path.join(file_name);
                fs::write(&filepath, ssz_snappy)
                    .context(anyhow!("Unable to write file {:?}", &filepath))?;
                modified_index = true;
            }
        }
    }
    Ok(modified_index)
}

/// For the given Unchained Index chunk files, finds transactions that match
/// The desired block range and address leading hex characters.
pub fn get_relevant_appearances(
    chunk_file_paths: Vec<&ChunkFile>,
    desired: BlockRange,
    leading_char: &str,
) -> Result<AddressIndexVolumeChapter> {
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
    let mut addresses: Vec<AddressAppearances> = relevant_appearances
        .into_iter()
        .map(|(key, val)| AddressAppearances {
            address: <_>::from(key),
            appearances: {
                // Start with Vec<TransactionId>
                // Get Vec<AppearanceTx>
                let t: Vec<AppearanceTx> = val
                    .iter()
                    .map(AppearanceTx::from_unchained_format)
                    .collect();
                // Get AppearanceTxList
                <_>::from(t)
            },
        })
        .collect();
    // Sort lexicographically by address. E.g., [0x0a, 0xa0, 0xaa].
    addresses.sort_by(|a, b| a.address.cmp(&b.address));

    let address_as_hex = hex::decode(leading_char)?;
    let res = AddressIndexVolumeChapter {
        address_prefix: <_>::from(address_as_hex),
        identifier: VolumeIdentifier {
            oldest_block: desired.old,
        },
        addresses: <_>::from(addresses),
    };
    Ok(res)
}

/// Represents metadata for a specific [chapter][1] directory containing [volume][2] files.
///
/// Used for creating file systems according to the [string naming conventions][3].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#addresschapter
/// [2]: https://github.com/perama-v/address-appearance-index-specs#addressindexvolumechapter
/// [3]: https://github.com/perama-v/address-appearance-index-specs#string-naming-conventions
pub struct ChapterDirectoryIdentifier {
    /// Common two hex characters at start of address in format. E.g., "1a".
    pub leading_chars: String,
    /// The directory includes files defined by ranges.
    pub volumes: Vec<BlockRange>,
}

/// Returns information about a [chapter][1] that can be used to construct a file system.
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#addresschapter
pub fn get_chapter_volumes(
    lowest_block_of_interest: u32,
    highest_block_in_chunks: u32,
) -> Result<Vec<ChapterDirectoryIdentifier>> {
    let mut target_files: Vec<ChapterDirectoryIdentifier> = vec![];
    let char_combinations = NUM_CHAPTERS;
    let volume_ids = complete_block_ranges(lowest_block_of_interest, highest_block_in_chunks)?;
    for n in 0..char_combinations {
        let chars: String = format!("{:0>2x}", n);
        let target_file = ChapterDirectoryIdentifier {
            leading_chars: chars,
            volumes: volume_ids.clone(),
        };
        target_files.push(target_file)
    }
    Ok(target_files)
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

/// For a given pair of block heights, returns a vector of `BlockRanges`, one for
/// every set of `RANGE` blocks.
///
/// Excludes new incomplete ranges. So the number of ranges is equal to the block
/// height divided by RANGE (integer division).
///
/// # Examples
///
/// ## Exclude latest
/// ```
/// # use min_know::transform::complete_block_ranges;
/// # use anyhow::{anyhow, Result};
///
/// let mut block_ranges = complete_block_ranges(0, 15_467_800)?;
/// assert_eq!(block_ranges.len(), 154);
///
/// let highest_range = block_ranges.pop()
///     .ok_or_else(|| anyhow!("Block height too low."))?;
/// assert_eq!(highest_range.old, 15_300_000);
/// assert_eq!(highest_range.new, 15_399_999);
/// # Ok::<(), anyhow::Error>(())
/// ```
/// In the example above, the `67_801` latest blocks are not included in the
/// returned ranges.
///
/// ## Exclude latest and first
/// Incomplete ranges below a lower bound are ignored.
/// In the example below, the oldest `100_000` (from range `[0, 99_999]`)
/// and newest `67_801` (from range `[15_400_000, 15_499_999]`) blocks are excluded.
/// ```
/// # use min_know::{
/// #     transform::complete_block_ranges,
/// #     constants::BLOCKS_PER_VOLUME
/// # };
///
/// let mut block_ranges = complete_block_ranges(100_000, 15_467_800)?;
/// assert_eq!(block_ranges.len(), 153);
/// # Ok::<(), anyhow::Error>(())
/// ```
/// ## Exclude latest and first two
/// In the example below, the oldest `200_000` (from ranges `[0, 99_999] and [100_000, 199_999]`)
/// and newest `67_801` (from range `[15_400_000, 15_499_999]`) blocks are excluded.
/// ```
/// # use anyhow::{anyhow, Result};
/// # use min_know::{
/// #     transform::complete_block_ranges,
/// #     constants::BLOCKS_PER_VOLUME
/// # };
///
/// let mut block_ranges = complete_block_ranges(200_000, 15_467_800)?;
/// assert_eq!(block_ranges.len(), 152);
/// let lowest_range = block_ranges.get(0)
///     .ok_or_else(|| anyhow!("Block height too low."))?;
/// assert_eq!(lowest_range.old, 200_000);
/// assert_eq!(lowest_range.new, 299_999);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn complete_block_ranges(
    oldest_desired_volume: u32,
    latest_height: u32,
) -> Result<Vec<BlockRange>> {
    if oldest_desired_volume % BLOCKS_PER_VOLUME != 0 {
        return Err(anyhow!("Must pass the first block in a volume."));
    }
    let n_groups = (latest_height + 1) / BLOCKS_PER_VOLUME;
    let mut ranges: Vec<BlockRange> = vec![];
    for i in 0..n_groups {
        let old_block = i * BLOCKS_PER_VOLUME;
        // Skip ranges that are outside the target range.
        let range_ok = old_block >= oldest_desired_volume;
        if !range_ok {
            continue;
        }
        let range = volume_id_to_block_range(old_block)?;
        ranges.push(range);
    }
    Ok(ranges)
}

/// Updates an existing address-appearance-index using additional Unchained
/// Index chunks.
///
/// Converts chunks to volume files if they do not already exist.
pub fn transform_missing_chunks(
    source: &UnchainedPath,
    destination: &AddressIndexPath,
    network: &Network,
) -> Result<bool> {
    let chunks_path = source.chunks_dir(network)?;
    let chunk_files: ChunksDir = ChunksDir::new(&chunks_path)?;
    let latest_mainnet = latest_block_in_chunks(&chunk_files)?;

    // Pass the identifier of the oldest missing volume.
    // E.g., if volume 14_400_000 is present, use volume 14_500_000.
    let first_missing_volume = destination.latest_volume(network)?.oldest_block + BLOCKS_PER_VOLUME;

    let chapter_dirs = get_chapter_volumes(first_missing_volume, latest_mainnet)?;

    let made_changes =
        create_specific_volume_files(destination, network, chapter_dirs, chunk_files)?;

    Ok(made_changes)
}

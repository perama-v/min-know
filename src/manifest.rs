//! Used to manage the address-appearance-index manifest.
//!
//! The manifest contains metadata about the index. This is
//! mainly versioning, and the hashes of the immutable contents
//! of the index. This enables auditing and acquisition of parts
//! of the index without holding the entire index.
//!
//! Furhter details can be read about in the [spec][1].
//!
//! [1]: https://github.com/perama-v/address-appearance-index-specs#indexmanifest
use anyhow::{anyhow, Context};
use ssz_types::FixedVector;
use std::{
    fs::{self, ReadDir},
    path::PathBuf,
    vec,
};
use tree_hash::TreeHash;

use crate::{
    constants::{
        ADDRESS_CHARS_SIMILARITY_DEPTH, BLOCK_RANGE_WIDTH, SPEC_VER_MAJOR, SPEC_VER_MINOR,
        SPEC_VER_PATCH, SPEC_RESOURCE_LOCATION, PUBLISHING_PREFIX,
    },
    encoding::decode_and_decompress,
    spec::{
        AddressIndexVolume, ChapterIdentifier, IndexManifest, ManifestChapter, ManifestVolume,
        NetworkName, VolumeIdentifier, IndexSpecificationVersion, IndexSpecificationSchemas, IndexPublishingIdentifier,
    },
    types::{AddressIndexPath, ChapterCompleteness, IndexCompleteness, Network},
    utils::{self},
};

/// Creates a new manifest file.
///
/// This will overwrite an existing manifest file.
///
/// ## Algorithm
/// Goes through each file in the data directory. Each one is
/// decompressed and the tree root hash is calculated. The values are
/// stored in memory. When all files are processed, the
/// data is serialized and compressed and written to a file called
/// "manifest.ssz_snappy" under the main data directory, alongside
/// the divisin folders.
pub fn generate(path: &AddressIndexPath, network: &Network) -> Result<(), anyhow::Error> {
    let chapters = get_chapter_dirs(path, network)?;
    let mut chapter_metadata: Vec<ManifestChapter> = vec![];
    let mut most_recent_volume: u32 = 0;
    // Only read valid directories.
    let chapters = chapters
        .filter_map(|x| x.ok())
        .filter(|f| {
            let name = f.file_name();
            match name.to_str() {
                Some(s) => s.starts_with("chapter_0x"),
                None => false,
            }
        })
        .enumerate();
    for (index, chapter) in chapters {
        let mut volume_metadata: Vec<ManifestVolume> = vec![];
        println!("Generating hash for chapter {} of n", index);
        let files = fs::read_dir(chapter.path())
            .with_context(|| format!("Failed to read dir: {:?}", &chapter.path()))?;
        for file in files {
            let volume_file = file?.path();
            let ssz_snappy_bytes = fs::read(&volume_file)
                .with_context(|| format!("Failed to read file: {:?}", &volume_file))?;
            let data: AddressIndexVolume = decode_and_decompress(ssz_snappy_bytes)?;
            let tree_hash_root = data.tree_hash_root();
            let identifier = VolumeIdentifier {
                oldest_block: data.identifier.oldest_block,
            };
            if data.identifier.oldest_block > most_recent_volume {
                most_recent_volume = data.identifier.oldest_block
            }
            let volume = ManifestVolume {
                identifier,
                tree_hash_root,
            };
            volume_metadata.push(volume);
        }
        let chap_name = &chapter.file_name();
        let chap_name = chap_name
            .to_str()
            .ok_or_else(|| anyhow!("div name {:?} not valid UTF-8", &chap_name))?;
        let chap_id = <_>::from(utils::chapter_dir_to_id(chap_name)?);
        volume_metadata.sort_by_key(|x| x.identifier.oldest_block);
        let chapter = ManifestChapter {
            identifier: ChapterIdentifier {
                address_common_bytes: chap_id,
            },
            volume_metadata: <_>::from(volume_metadata),
        };
        chapter_metadata.push(chapter)
    }
    chapter_metadata.sort_by(|a, b| {
        b.identifier
            .address_common_bytes
            .cmp(&a.identifier.address_common_bytes)
    });
    let manifest = IndexManifest {
        version: IndexSpecificationVersion {
            major: SPEC_VER_MAJOR,
            minor: SPEC_VER_MINOR,
            patch: SPEC_VER_PATCH
        },
        publish_as_topic:  IndexPublishingIdentifier {
            topic: {
                let topic_string = format!("{}{}", PUBLISHING_PREFIX, network.name());
                <_>::from(topic_string.as_bytes().to_vec())
            }
        },
        schemas: IndexSpecificationSchemas {
            resource: <_>::from(SPEC_RESOURCE_LOCATION.as_bytes().to_vec())
        },
        network: NetworkName {
            name: <_>::from(network.name().as_bytes().to_vec()),
        },
        latest_volume_identifier: VolumeIdentifier {
            oldest_block: most_recent_volume,
        },
        chapter_metadata: FixedVector::from(chapter_metadata),
    };
    let manifest_name = manifest.file_name_no_encoding()?;

    // Make JSON manifest file.
    let json_manifest = serde_json::to_string_pretty(&manifest)?;
    let mut json_filename = path.index_dir(network)?.join(PathBuf::from(&manifest_name));
    json_filename.set_extension("json");
    fs::write(&json_filename, json_manifest)
        .with_context(|| format!("Failed to write file: {:?}", &json_filename))?;
    Ok(())
}

/// Retrieves the contents of the index manifest
///
/// The manifest is stored as JSON.
/// This extracts the manifest in a readable form.
///
/// ## Example
/// ```
/// use min_know::{manifest, types::{AddressIndexPath, Network}};
///
/// let data_dir = AddressIndexPath::Sample;
/// let network = Network::default();
/// let manifest = manifest::read(&data_dir, &network)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
/// ## Errors
/// The index directory will be searched for the manifest file and the spec
/// version extracted from the file name. Will `Error` in the event of a major
/// incompatibility with the spec version in the library.
pub fn read(path: &AddressIndexPath, network: &Network) -> Result<IndexManifest, anyhow::Error> {
    let filename = path.manifest_file(network)?;
    let json_format =
        fs::read(&filename).with_context(|| format!("Failed to read file: {:?}", &filename))?;
    let manifest: IndexManifest = serde_json::from_slice(&json_format)?;
    if manifest.version.major != SPEC_VER_MAJOR {
        return Err(anyhow!(
            "The manifest major version (v{}.x.x) is different from the spec version
        for this libray (v{}.x.x).",
            manifest.version.major,
            SPEC_VER_MAJOR
        ));
    }
    let n1 = manifest.network_name()?;
    let n2 = network.name().to_owned();
    if n1 != n2 {
        return Err(anyhow!(
            "The manifest network ({}) is different from expected ({}).",
            n1,
            n2
        ));
    }
    Ok(manifest)
}

/// Gets chapter directories from index path.
///
/// ## Errors
/// Only used on complete index data (for generating a manifest).
/// Errors if not using sample data and there are missing chapter directories.
fn get_chapter_dirs(path: &AddressIndexPath, network: &Network) -> Result<ReadDir, anyhow::Error> {
    let index = path.index_dir(network)?;
    match path {
        AddressIndexPath::Sample => {}
        _ => {
            let expected_dirs = 16_usize.pow(ADDRESS_CHARS_SIMILARITY_DEPTH);
            let chap_dirs = fs::read_dir(&index)
                .with_context(|| format!("Failed to read dir: {:?}", &index))?;
            if chap_dirs.count() < expected_dirs {
                return Err(anyhow!(
                    "Should not generate a manifest on incomplete data."
                ));
            }
        }
    }
    fs::read_dir(&index).with_context(|| format!("Failed to read dir: {:?}", &index))
}

/// Checks local data against the contents of the manifest.
///
/// Returns a report that can be used to assess completeness of local data.
///
/// ## Algorithm
///
/// Read the manifest to get chapters and their volumes. Try to read
/// the corresponding volume files and compute the ssz root hash.
///
/// Record the result (ok, absent, bad_hash). If a chapter has complete
/// set of volumes, the hashes are not checked.
pub fn completeness_audit(
    index_path: &AddressIndexPath,
    network: &Network,
) -> Result<IndexCompleteness, anyhow::Error> {
    let manifest = read(index_path, network)?;
    let volumes_per_chapter = manifest.latest_volume_identifier.oldest_block / BLOCK_RANGE_WIDTH;
    let mut audit = IndexCompleteness {
        complete_chapters: vec![],
        incomplete_chapters: vec![],
        absent_chapters: vec![],
    };
    for manifest_chapter in manifest.chapter_metadata.iter() {
        println!(
            "Checking chapter {}.",
            manifest_chapter.identifier.as_string()
        );
        let chap_str = manifest_chapter.identifier.as_string();
        let chap_path = index_path.chapter_dir(network, &chap_str)?;
        match fs::read_dir(chap_path) {
            Ok(chap_files) => {
                if *&chap_files.count() as u32 == volumes_per_chapter {
                    // If dir content count matches expected volumes per chapter.
                    audit
                        .complete_chapters
                        .push(manifest_chapter.identifier.clone())
                } else {
                    // Else, document incomplete chapter.
                    let comp = get_chapter_completeness(
                        index_path,
                        network,
                        &manifest_chapter,
                        &chap_str,
                    )?;
                    audit.incomplete_chapters.push(comp);
                }
            }
            Err(_) => {
                // dir is absent, add to absent list.
                audit
                    .absent_chapters
                    .push(manifest_chapter.identifier.clone())
            }
        }
    }
    Ok(audit)
}

/// For a given chapter in the manifest, finds which of its volumes
/// are present in the associated data directory.
///
/// The `chap_str` is of the form "5e".
pub fn get_chapter_completeness(
    index_path: &AddressIndexPath,
    network: &Network,
    div: &ManifestChapter,
    chap_str: &str,
) -> Result<ChapterCompleteness, anyhow::Error> {
    let mut c = ChapterCompleteness {
        id: div.identifier.clone(),
        ok: vec![],
        absent: vec![],
        bad_hash: vec![],
    };

    for volume in div.volume_metadata.iter() {
        volume.identifier.oldest_block;
        let volume_path =
            index_path.volume_file(network, chap_str, volume.identifier.oldest_block)?;

        match fs::read(volume_path) {
            Ok(file) => {
                let data: AddressIndexVolume = decode_and_decompress(file)?;
                let hash = data.tree_hash_root();
                if hash != volume.tree_hash_root {
                    // Incorrect hash.
                    c.bad_hash.push(volume.identifier)
                } else {
                    // Correct hash.
                    c.ok.push(volume.identifier)
                }
            }
            Err(_) => {
                // File missing.
                c.absent.push(volume.identifier)
            }
        }
    }

    Ok(c)
}

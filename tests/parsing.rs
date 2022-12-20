use std::{fs, path::PathBuf};

use anyhow::Context;
use min_know::specs::{
    address_appearance_index::{AAIAppearanceTx, AAIChapterId, AAISpec, AAIVolumeId},
    traits::{ChapterIdMethods, VolumeIdMethods},
};

use crate::common::aai_db;

mod common;

#[test]
fn index_dir_readable() {
    let dir = fs::read_dir(aai_db().config.data_dir).unwrap();
    // 256 chapters.
    assert_eq!(dir.count(), 256);
}

#[test]
fn uc_files_present() {
    let dir = fs::read_dir(aai_db().config.raw_source).unwrap();
    // 5 Unchained Index sample files.
    assert_eq!(dir.count(), 5);
}

#[test]
fn sample_files_all_greater_than_50kb() {
    let db = aai_db();
    let chapter_dirs = fs::read_dir(&db.config.data_dir)
        .with_context(|| format!("Couldn't read data directory {:?}.", &db.config.data_dir))
        .unwrap();
    for chapter_dir in chapter_dirs {
        // Obtain ChapterId from directory name.
        let dir = chapter_dir.unwrap().path();
        let chap_id = AAIChapterId::from_chapter_directory(&dir).unwrap();
        // Obtain VolumeIds using ChapterId
        let chapter_files: Vec<(PathBuf, AAIVolumeId)> = db
            .config
            .parse_all_files_for_chapter::<AAISpec>(&chap_id)
            .unwrap();
        for (chapter_path, _volume_id) in chapter_files {
            let bytes = fs::read(chapter_path).unwrap();
            let kbytes = bytes.len() / 1000;
            assert_eq!(kbytes > 50, true);
        }
    }
}

#[test]
fn sample_manifest_readable() {
    aai_db().manifest().unwrap();
}

#[test]
fn skips_incomplete_volumes() {
    let manifest = aai_db().manifest().unwrap();
    // Chunk 15_508_866 should be skipped until a chunk
    // including 15_599_999 is present.
    let volume = AAIVolumeId::from_interface_id(&manifest.latest_volume_identifier).unwrap();
    assert_eq!(volume.oldest_block, 14_400_000);
}

#[test]
fn detects_known_txs() {
    // EF dev wallet with known txs in the sample data.
    let known_count = 53;
    let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
    let db = aai_db();
    let values = db.find(address).unwrap();
    let mut appearances: Vec<AAIAppearanceTx> = vec![];
    for v in values {
        appearances.extend(v.value.to_vec());
    }
    assert_eq!(known_count, appearances.len());
}

use std::fs;

use min_know::specs::{address_appearance_index::{AAIVolumeId, AAIAppearanceTx}, traits::VolumeIdMethods};

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
fn sample_files_nonzero() {
    todo!("Ensure that sample files all contain data.");
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

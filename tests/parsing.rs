use std::fs;

use min_know::{
    constants::BLOCKS_PER_VOLUME,
    discover,
    fetch::{appearance_index_samples_present, unchained_samples_present},
};

mod common;

#[test]
fn index_dir_readable() {
    let dir = fs::read_dir(common::index_dir()).unwrap();
    // 256 chapter and a manifest.
    assert_eq!(dir.count(), 257);
}

#[test]
fn files_present() {
    let (data_dir, _, network) = common::dir_and_network();
    appearance_index_samples_present(&data_dir, &network).unwrap();
}

#[test]
fn uc_files_present() {
    let (_, data_dir, network) = common::dir_and_network();
    unchained_samples_present(&data_dir, &network).unwrap();
}


#[test]
fn sample_manifest_readable() {
    common::manifest();
}

#[test]
fn skips_incomplete_volumes() {
    let manifest = common::manifest();
    // Chunk 15_508_866 should be skipped until a chunk
    // including 15_599_999 is present.
    assert_eq!(manifest.latest_volume_identifier.oldest_block, 14_400_000);
}

#[test]
fn detects_known_txs() {
    // EF dev wallet with known txs in the sample data.
    let known_count = 53;
    let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
    let (data_dir, _, network) = common::dir_and_network();
    let appearances = discover::single_address(address, &data_dir, &network).unwrap();
    assert_eq!(known_count, appearances.len());
}

#[test]
fn tx_in_correct() {
    let (data_dir, _, network) = common::dir_and_network();
    let volume_oldest = 12_300_000;
    let chapter = "ff";

    let volume_path = data_dir
        .volume_file(&network, chapter, volume_oldest)
        .unwrap();
    let volume = discover::single_volume(volume_path).unwrap();
    let add_app = volume.addresses.get(45).unwrap();
    let address = hex::encode(&add_app.address.to_vec());
    let appearances = &add_app.appearances;
    let first = appearances.get(0).unwrap();

    assert_eq!(first.block >= volume_oldest, true);
    assert_eq!(first.block < volume_oldest + BLOCKS_PER_VOLUME, true);
    assert_eq!(address.starts_with(chapter), true);
}

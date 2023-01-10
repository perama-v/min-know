use std::{fs, path::PathBuf};

use anyhow::Context;
use min_know::specs::{
    nametags::{NameTagsChapterId, NameTagsSpec, NameTagsVolumeId},
    traits::ChapterIdMethods,
};

use crate::common::nametags_db;

#[test]
fn index_dir_readable() {
    let path = dbg!(nametags_db().config.data_dir);
    let dir = fs::read_dir(path).unwrap();
    // 256 chapters.
    assert_eq!(dir.count(), 256);
}

#[test]
fn source_files_present() {
    let path = dbg!(nametags_db().config.raw_source);
    let dir = fs::read_dir(path).unwrap();
    // 2063 raw nametag sample files expected.
    assert_eq!(dir.count(), 2063);
}

#[test]
fn sample_files_all_greater_than_5_bytes() {
    let db = nametags_db();
    let chapter_dirs = fs::read_dir(&db.config.data_dir)
        .with_context(|| format!("Couldn't read data directory {:?}.", &db.config.data_dir))
        .unwrap();
    for chapter_dir in chapter_dirs {
        // Obtain ChapterId from directory name.
        let dir = chapter_dir.unwrap().path();
        let chap_id = NameTagsChapterId::from_chapter_directory(&dir).unwrap();
        // Obtain VolumeIds using ChapterId
        let chapter_files: Vec<(PathBuf, NameTagsVolumeId)> = db
            .config
            .parse_all_files_for_chapter::<NameTagsSpec>(&chap_id)
            .unwrap();
        for (chapter_path, _volume_id) in chapter_files {
            let bytes = fs::read(chapter_path).unwrap();
            assert_eq!(bytes.len() > 5, true);
        }
    }
}

#[test]
fn sample_manifest_readable() {
    nametags_db().manifest().unwrap();
}

#[test]
fn detects_known_nametags() {
    // EF dev wallet with known tags in the sample data.
    let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
    let db = nametags_db();
    let values = db.find(address).unwrap();
    let mut names = vec![];
    let mut tags = vec![];
    for v in values {
        names.extend(v.names_as_strings().unwrap());
        tags.extend(v.tags_as_strings().unwrap());
    }
    let expected_names = vec!["EthDev"];
    assert_eq!(expected_names, names);
    let expected_tags = vec!["ethereum-foundation"];
    assert_eq!(expected_tags, tags);
}

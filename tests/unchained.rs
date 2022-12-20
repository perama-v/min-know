use std::path::PathBuf;

use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};
use min_know::utils::unchained::types::{BlockRange, UnchainedFile};

mod common;

#[test]
fn sample_header_sample_ok() {
    let db = common::aai_db();
    let index_path = db.config.raw_source.join("011283653-011286904.bin");
    let target = BlockRange {
        old: 11_200_000,
        new: 11_300_000,
    };
    let _uf = UnchainedFile::new(index_path, target).unwrap();
}

#[test]
fn sample_header_local_ok() {
    println!("Env is: {:?}", std::env::current_dir());
    // Run test from this dir:
    let db = common::aai_db();
    let local_example_dir_raw =
        PathBuf::from("./data/samples").join(db.config.data_kind.raw_source_dir_name());
    // Look for this file:
    let index_path = local_example_dir_raw.join("011283653-011286904.bin");
    let target = BlockRange {
        old: 11_200_000,
        new: 11_300_000,
    };
    let _uf = UnchainedFile::new(index_path, target).unwrap();
}

#[test]
fn cid_in_out() {
    let h = Code::Sha2_256.digest(b"abcd1234");
    let cid = Cid::new_v0(h).unwrap();
    let data = cid.to_bytes();
    let out = Cid::try_from(data).unwrap();
    assert_eq!(cid, out);
}

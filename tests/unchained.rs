use std::path::PathBuf;

use min_know::{
    types::{Network, UnchainedPath},
    unchained::types::{BlockRange, UnchainedFile},
};
use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};

mod common;

#[test]
fn sample_header_sample_ok() {
    let path = UnchainedPath::Sample;
    let network = Network::default();
    let index_path = path
        .chunks_dir(&network)
        .unwrap()
        .join("011283653-011286904.bin");
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
    let path = UnchainedPath::Custom(PathBuf::from("./data"));
    let network = Network::default();
    // Look for this file:
    let index_path = path
        .chunks_dir(&network)
        .unwrap()
        .join("011283653-011286904.bin");
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

use std::path::PathBuf;

use min_know::{spec::IndexManifest, types::*, *};
use anyhow::Context;

pub fn dir_and_network() -> (AddressIndexPath, UnchainedPath, Network) {
    (
        AddressIndexPath::Sample,
        UnchainedPath::Sample,
        Network::default(),
    )
}

/// Setup for paths.
pub fn index_dir() -> PathBuf {
    let data_dir = AddressIndexPath::Sample;
    let network = Network::default();
    data_dir
        .index_dir(&network)
        .with_context(|| format!("Could not read directory for {:?}", network))
        .unwrap()
}

/// Setup for integration tests.
pub fn manifest() -> IndexManifest {
    let data_dir = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&data_dir, &network);
    index.read_manifest().unwrap()
}

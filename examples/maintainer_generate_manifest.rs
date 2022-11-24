use std::env;

use anyhow::Result;

use min_know::{
    types::{AddressIndexPath, Network, UnchainedPath},
    IndexConfig,
};


/// Creates the index manifest.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    let path = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&path, &network);

    // Create the new manifest.
    index.maintainer_generate_manifest()?;
    Ok(())
}

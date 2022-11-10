use min_know::{
    types::{AddressIndexPath, Network, UnchainedPath},
    IndexConfig,
};
use std::env;

/// Creates the index manifest.
fn main() -> Result<(), anyhow::Error> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    let path = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&path, &network);

    // Create the new manifest.
    index.maintainer_generate_manifest()?;
    Ok(())
}

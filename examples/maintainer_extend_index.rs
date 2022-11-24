use std::env;

use anyhow::Result;

use min_know::{
    types::{AddressIndexPath, Network, UnchainedPath},
    IndexConfig,
};

/// Uses local data to take recent parts of the Unchained Index and add them
/// to the address-appearance-index.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    let path = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&path, &network);

    // Update the index and manifest.
    let unchained_path = UnchainedPath::Sample;
    index.maintainer_extend_index(&unchained_path)?;

    Ok(())
}

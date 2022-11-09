use min_know::{
    types::{AddressIndexPath, Network, UnchainedPath},
    IndexConfig,
};
use std::env;

/// Uses local data to take recent parts of the Unchained Index and add them
/// to the address-appearance-index.
fn main() -> Result<(), anyhow::Error> {
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

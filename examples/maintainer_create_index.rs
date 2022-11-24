use std::env;

use anyhow::Result;

use min_know::{
    types::{AddressIndexPath, Network, UnchainedPath},
    IndexConfig,
};
/// Creates the index using local data by fetching the Unchained Index and duplicating
/// the data in a different format suitable for chapter and distribution.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    let path = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&path, &network);

    // Create the new index and manifest.
    let unchained_path = UnchainedPath::Sample;
    index.maintainer_create_index(&unchained_path)?;

    Ok(())
}

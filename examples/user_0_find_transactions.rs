use std::env;

use anyhow::Result;

use min_know::{
    types::{AddressIndexPath, Network},
    IndexConfig,
};

/// Uses local index data to extract transaction identifiers important for a given address.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    // An address. E.g., EF dev wallet.
    let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
    // Another random address.
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9";
    // Choose between real or sample data.
    let data_dir = AddressIndexPath::Sample;
    // Default is mainnet.
    let network = Network::default();
    let index = IndexConfig::new(&data_dir, &network);

    let manifest = index.read_manifest()?;

    println!(
        "The manifest for this data has data for volumes up to and including {}",
        manifest.latest_volume_identifier.oldest_block
    );

    let appearances = index.find_transactions(address)?;

    println!(
        "Txs {:#?}\n\nAddress {:?} appeared during the execution of {} transactions",
        appearances,
        address,
        appearances.len(),
    );
    Ok(())
}

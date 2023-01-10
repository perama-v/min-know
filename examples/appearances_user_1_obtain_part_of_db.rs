use std::env;

use anyhow::{bail, Result};

use min_know::{
    config::{
        address_appearance_index::Network,
        choices::{DataKind, DirNature},
    },
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
};

/// Uses a manifest file to obtain data relevant for a user.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let data_kind = DataKind::AddressAppearanceIndex(Network::default());
    let db: Todd<AAISpec> = Todd::init(data_kind, DirNature::Sample)?;

    // Addresses important for a user (two random addresses).
    let addresses = [
        // Random address.
        "0x846be97d3bf1e3865f3caf55d749864d39e54cb9",
        // EF dev wallet.
        "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae",
    ];
    static IPFS_GATEWAY_URL: &str = "https://127.0.0.1:8080";

    // Obtain Chapters with ChapterIds: 0x84 and 0xde
    db.obtain_relevant_data(&addresses, IPFS_GATEWAY_URL)?;

    let Some(address) = addresses.get(0) else { bail!("Address not in list.")};
    let appearances = db.find(address)?;
    for a in appearances {
        println!("Appearance Tx ID: {:?}", a);
    }
    Ok(())
}

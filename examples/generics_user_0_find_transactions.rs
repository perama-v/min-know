use std::env;

use anyhow::Result;

use min_know::{
    config::{
        address_appearance_index::Network,
        dirs::{DataKind, DirNature},
    },
    database::types::Todd,
    specs::{
        address_appearance_index::AAISpec,
        types::{DataSpec, SpecId},
    },
};
use ssz::Encode;

/// Uses local index data to extract transaction identifiers important for a given address.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    // A random address.
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9";
    let db: Todd<AAISpec> = Todd::new(DataKind::default(), DirNature::Sample)?;

    println!(
        "DB is {:#?}, with name {} and num chapters {}",
        db,
        db.spec_name(),
        db.chapters.len()
    );
    let appearances = db.find(address)?;
    for a in appearances {
        println!("Appearance Tx ID: {:?}", a);
    }
    Ok(())
}

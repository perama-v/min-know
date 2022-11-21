use std::env;

use min_know::{
    database::types::Todd,
    config::address_appearance_index::Config,
    specs::address_appearance_index::{AdApInSpec}
};

/// Uses local index data to extract transaction identifiers important for a given address.
fn main() -> Result<(), anyhow::Error> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    // A random address.
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9";

    let _config = Config {};
    let db = Todd::<AdApInSpec>::new();

    println!(
        "DB is {:#?}, with name {} and num units {}",
        db,
        db.spec_name(),
        db.units.len()
    );

    // let appearances = db.find_transactions(address)?;
    // Find transactions becomes db.read_qurey(address)
    let appearances = db.read_query(address);
    assert
    Ok(())
}

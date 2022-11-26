use std::env;

use anyhow::Result;

use min_know::{
    config::dirs::DirNature,
    database::types::Todd,
    specs::{address_appearance_index::{AdApInSpec}, types::{SpecId, DataSpec}},
};
use ssz::Encode;

/// Uses local index data to extract transaction identifiers important for a given address.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    // A random address.
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9";

    let db: Todd<AdApInSpec> = Todd::new(SpecId::AddressAppearanceIndex, DirNature::Sample)?;

    println!(
        "DB is {:#?}, with name {} and num chapters {}",
        db,
        db.spec_name(),
        db.chapters.len()
    );

    // let appearances = db.find_transactions(address)?;
    // Find transactions becomes db.read_qurey(address)
    let appearances: <AdApInSpec as DataSpec>::AssociatedRecordValue = db.read_record_key(address)?;
    let address_found = hex::encode(appearances.record_key.as_ssz_bytes());
    assert_eq!(address_found, address);
    Ok(())
}

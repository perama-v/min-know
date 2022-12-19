use std::env;

use anyhow::Result;

use min_know::{
    config::dirs::{DataKind, DirNature},
    database::types::Todd,
    specs::{address_appearance_index::{AAISpec, AAIAppearanceTx}},
};

/// Uses local index data to extract transaction identifiers important for a given address.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample)?;
    println!("DB is {:#?}", db);

    // A random address.
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9";
    let values = db.find(address)?;
    let mut appearances: Vec<AAIAppearanceTx> = vec![];
    for v in values {
        appearances.extend(v.value.to_vec());
    }
    println!("{:?}",appearances);
    Ok(())
}

use std::env;

use anyhow::{bail, Result};

use min_know::{
    config::choices::{DataKind, DirNature},
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
};

/// Uses a manifest file to obtain data relevant for a user.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample)?;
    println!("DB is {:#?}", db);

    let check = db.check_completeness()?;
    println!("Check result: {:?}", check);
    Ok(())
}

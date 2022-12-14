use std::env;

use anyhow::Result;

use min_know::{
    config::dirs::{DataKind, DirNature},
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
};
/// Creates the index using local data.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let mut db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample)?;
    db.generate_manifest()?;

    Ok(())
}

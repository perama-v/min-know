use std::env;

use anyhow::Result;

use min_know::{
    config::choices::{DataKind, DirNature},
    database::types::Todd,
    specs::signatures::SignaturesSpec,
};

/// Creates the database using local data.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "0");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db: Todd<SignaturesSpec> = Todd::init(DataKind::Signatures, DirNature::Sample)?;
    db.full_transformation()?;

    Ok(())
}

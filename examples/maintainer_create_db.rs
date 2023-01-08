use std::env;

use anyhow::Result;

use min_know::{
    config::{choices::{DataKind, DirNature}, address_appearance_index::Network},
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
};
/// Creates the index using local data.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let data_kind = DataKind::AddressAppearanceIndex(Network::default());
    let db: Todd<AAISpec> = Todd::init(data_kind, DirNature::Sample)?;
    db.full_transform()?;

    Ok(())
}

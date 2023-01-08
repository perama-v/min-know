use std::env;

use anyhow::Result;
use env_logger;

use min_know::{
    config::{choices::{DataKind, DirNature}, address_appearance_index::Network},
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
};

/// Obtains/downloads sample index data that can be used for testing.
///
/// Try the following examples next:
/// ```bash
/// cargo run --example user_0_find_transactions_generic
/// cargo run --example user_check_completeness_generic
/// ```
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let data_kind = DataKind::AddressAppearanceIndex(Network::default());
    let db: Todd<AAISpec> = Todd::init(data_kind, DirNature::Sample)?;
    db.get_sample_data()?;
    Ok(())
}

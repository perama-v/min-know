use std::env;

use anyhow::Result;

use min_know::{
    config::dirs::{DataKind, DirNature},
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
};

/// Downloads sample index data that can be used for testing.
///
/// Try the following examples next:
/// ```bash
/// cargo run --example generics_user_0_find_transactions
/// cargo run --example generics_user_check_completeness
/// ```
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    let db: Todd<AAISpec> = Todd::new(DataKind::default(), DirNature::Sample)?;
    db.get_sample_data()?;
    Ok(())
}

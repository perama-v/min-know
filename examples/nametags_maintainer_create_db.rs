use std::env;

use anyhow::Result;

use min_know::{
    config::choices::{DataKind, DirNature},
    database::types::Todd,
    specs::nametags::NameTagsSpec,
};
/// Creates the database using local data.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "0");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let data_kind = DataKind::NameTags;
    let db: Todd<NameTagsSpec> = Todd::init(data_kind, DirNature::Sample)?;
    db.full_transform()?;

    Ok(())
}

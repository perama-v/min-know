use std::env;

use anyhow::Result;

use min_know::{config::choices::{DataKind, DirNature}, database::types::Todd, specs::nametags::NameTagsSpec};

fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db: Todd<NameTagsSpec> = Todd::init(DataKind::NameTags, DirNature::Sample)?;
    db.get_sample_data()?;
    Ok(())
}

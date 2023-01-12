use std::{env, fs::canonicalize, path::PathBuf};

use anyhow::Result;

use min_know::{
    config::choices::{DataKind, DirNature, PathPair},
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
    let rolo = canonicalize(PathBuf::from("../../../Repos/RolodETH/data"))?;
    let paths = PathPair {
        raw_source: Some(rolo),
        processed_data_dir: None,
    };
    let db: Todd<NameTagsSpec> = Todd::init(data_kind, DirNature::Custom(paths))?;

    db.full_transformation()?;

    Ok(())
}

use std::env;

use anyhow::Result;

use min_know::{
    config::choices::{DataKind, DirNature},
    database::types::Todd,
    specs::signatures::SignaturesSpec,
};

/// Uses local index data to extract transaction identifiers important for a given address.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let db: Todd<SignaturesSpec> = Todd::init(DataKind::Signatures, DirNature::Sample)?;

    let signature = "ddf252ad"; // Transfer(address,address,uint256)
                                //let signature = "e1fffcc4"; // Deposit(address,uint256)

    let text = db.find(signature)?;
    for t in text {
        println!("{}", t);
    }

    Ok(())
}

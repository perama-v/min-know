use std::env;

use anyhow::Result;

use min_know::{
    types::{AddressIndexPath, Network},
    IndexConfig,
};
/// Uses a table of hashes to check if local index data matches that which is expected by the user.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    let data_dir = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&data_dir, &network);

    let check = index.check_completeness()?;
    println!(
        "Local data contains: \n\tComplete: {}\n\tAbsent: {}\n\tIncomplete: {}",
        check.complete_chapters.len(),
        check.absent_chapters.len(),
        check.incomplete_chapters.len()
    );
    if check.incomplete_chapters.len() > 0 {
        let first = check.incomplete_chapters.get(0).unwrap();
        println!(
            "E.g., chapter 0x{} contains: \n\tOk: {} \n\tAbsent: {} \n\tIncorrect hash: {}",
            first.id.as_string(),
            first.ok.len(),
            first.absent.len(),
            first.bad_hash.len()
        )
    }

    Ok(())
}

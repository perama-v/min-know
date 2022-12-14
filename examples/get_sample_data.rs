use std::env;

use anyhow::Result;

use min_know::{
    types::{AddressIndexPath, Network, UnchainedPath},
    IndexConfig,
};

/// Downloads sample index data that can be used for testing.
///
/// Try the following examples next:
/// ```bash
/// cargo run --example user_find_transactions
/// cargo run --example user_check_completeness
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    let data_dir = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&data_dir, &network);
    let unchained_path = UnchainedPath::Sample;

    index.get_sample_data(&unchained_path).await?;
    Ok(())
}

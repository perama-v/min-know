//! This library provides utilities for creating, using and maintaining an index that
//! maps Ethereum addresses to the historical transactions they appear in.
//!
//! The address-appearance-index ([spec][0]) is a derivative of the
//! Unchained Index ([paper][1]/[docs][2]/[repo][3]). Data is organised at the
//! address level, rather than at the block level. A user can obtain a fraction
//! of the index that is relevant for a particular address.
//! This allows a resource constrained user to obtain knowledge
//! from peers of where their address has appeared historically.
//!
//! [0]: https://github.com/perama-v/address-appearance-index-specs
//! [1]: https://trueblocks.io/papers/2022/file-format-spec-v0.40.0-beta.pdf
//! [2]: https://trueblocks.io/docs/
//! [3]: https://github.com/TrueBlocks/trueblocks-core
//! ---
//! # Use index (small portal node)
//! A user with wallet address `0xde0b...7bae` obtains the `0xde` piece of the
//! index from a peer. Then they can extract transaction data locally as follows:
//!
//! ```
//! use std::path::PathBuf;
//! use min_know::{
//!     discover::single_address,
//!     IndexConfig,
//!     types::{AddressIndexPath, Network}};
//! use anyhow::{anyhow, Result};
//!
//! let data_dir = AddressIndexPath::Sample;
//! let network = Network::default();
//! let index = IndexConfig::new(&data_dir, &network);
//!
//! // Get transaction information.
//! let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
//! let tx_info = index.find_transactions(address)?;
//! let first_tx = tx_info.get(0).ok_or_else(||
//!     anyhow!("No transactions found for this address."))?;
//!
//! // The block and index of each transaction for that address is available.
//! assert_eq!(first_tx.block, 12_387_430);
//! assert_eq!(first_tx.index, 140);
//! # Ok::<(), anyhow::Error>(())
//!```
//!
//! Example: [address-appearance-index/examples/user_find_transactions.rs](../../../address_appearance_index/examples/user_find_transactions.rs).
//!
//! ```sh
//! cargo run --example user_find_transactions
//! ```
//!
//! # User completeness audit (small portal node)
//!
//! Given a volume of the address-appearance-index, check that the hashes
//! of the local content match those that appear in the
//! address-appearance-index-manifest.
//!
//! This checks that the local data is:
//! - Complete (data list matches manifest).
//! - Correct (data hashes match manifest).
//!
//! Additional data can then be fetched from portal node peers on the
//! address-appaearance-index network.
//!
//! Example: [address-appearance-index/examples/user_check_completeness.rs](../../../address_appearance_index/examples/user_check_completeness.rs).
//!
//! ```sh
//! cargo run --example user_check_completeness
//! ```
//!
//! # Maintainer create index (large portal bridge node)
//!
//! Convert the entire Unchained Index into the address-appearance-index.
//!
//! Code: [address-appearance-index/examples/maintainer_create_index.rs](../../../maintainer_create_index.rs).
//!
//! ```sh
//! cargo run --example maintainer_create_index
//! ```
//!
//! # Maintainer extend index (large portal bridge node)
//!
//! Start with an out-of-date address-appearance-index and newer Unchained Index chunks.
//! Convert the chunks and incorporate into the address-appearance-index.
//!
//! 1. address-appearance-index complete up to a certain block height.
//! 2. Tracing archive execution node
//! 3. TrueBlocks `chifra scrape` to update Unchained Index
//! 4. Use this library to fetch latest data from Unchained Index and convert into
//! address-appearance-index.
//!
//! Code: [address-appearance-index/examples/maintainer_extend_index.rs](../../../maintainer_extend_index.rs).
//!
//! ```sh
//! cargo run --example maintainer_extend_index
//! ```
//!
//! # Maintainer correctness audit (large portal bridge node)
//!
//! Given an address-appearance-index and an Unchained Index, randomly
//! cross check samples from the indices against each other. Discrepancies
//! are logged for further analysis. An error indicates that the parsing
//! performed by the bridge node is incorrect.
//!
//! Code: [address-appearance-index/examples/maintainer_audit_correctness.rs](../../../maintainer_audit_correctness.rs).
//!
//! ```sh
//! cargo run --example maintainer_audit_correctness
//! ```
//!
//! ---
//! # Additional information
//!
//! The address-apppearance-index
//! [specification](https://github.com/perama-v/address-appearance-index-specs).
//!
//! ## Resources
//! A user might expect to use <500MB for each address they wish to examine.
//! For comparison, the full Unchained Index is ~80GB, and is perfect for more
//! comprehensive analyses when paired with a full node.
//!
//! ## Users
//! The address-appearance-index could work well with a [Portal Network][1], which seeks to be a peer to peer network for resource constrained devices. Portal client implementations:
//! - [Trin][2] Rust client by the Ethereum Foundation
//! - [Fluffy][3] Nim client by the Nimbus team.
//! - [Ultralight][4] Typescript client by the EthereumJS team.
//!
//! After discovering which transactions are of interest, a Portal Network user could fetch
//! them using the various overlay networks. The address-appearance-index might be suited to
//! be an overlay network itself.
//!
//! [1]: https://github.com/ethereum/portal-network-specs
//! [2]: https://github.com/ethereum/trin
//! [3]: https://github.com/status-im/nimbus-eth1/tree/master/fluffy
//! [4]: https://github.com/ethereumjs/ultralight
pub mod common;
pub mod config;
pub mod database;
pub mod extraction;
pub mod parameters;
pub mod samples;
pub mod specs;

pub mod constants;
pub mod contract_utils;
pub mod discover;
pub mod encoding;
pub mod fetch;
pub mod ipfs;
pub mod manifest;
pub mod spec;
pub mod transform;
pub mod types;
pub mod unchained;
pub mod utils;

use anyhow::Result;

use constants::BLOCKS_PER_VOLUME;
use spec::{AppearanceTx, IndexManifest};
use types::{AddressIndexPath, IndexCompleteness, Network, UnchainedPath};

/// Configures the library parameters and provides a simple interface for common commands.
///
/// Parameter specification mainly involves specifying where data is located, and
/// what sort of network parameters are being used.
///
/// ## Example
/// Commands can be found by using tab-suggestions from `index.`
/// ```
/// use min_know::{IndexConfig, types::{AddressIndexPath, Network}};
///
/// let data_dir = AddressIndexPath::Sample;
/// let network = Network::default();
/// let index = IndexConfig::new(&data_dir, &network);
/// let manifest = index.read_manifest()?;
/// # Ok::<(), anyhow::Error>(())
///
/// ```
/// See `./examples/` dir for more.
pub struct IndexConfig {
    path: AddressIndexPath,
    network: Network,
}

impl IndexConfig {
    /// Configures the library parameters and provides a simple interface for common commands.
    pub fn new(path: &AddressIndexPath, network: &Network) -> Self {
        IndexConfig {
            path: path.clone(),
            network: network.clone(),
        }
    }
    /// Creates the full address-appearance-index using the Unchained Index.
    /// Creates a new manifest file.
    pub fn maintainer_create_index(&self, source: &UnchainedPath) -> Result<()> {
        transform::full_transform(source, &self.path, &self.network)?;
        manifest::generate(&self.path, &self.network)?;
        Ok(())
    }
    /// Updates an existing address-appearance-index using additional Unchained Index chunks.
    /// Generates a new manifest if updates are made.
    pub fn maintainer_extend_index(&self, source: &UnchainedPath) -> Result<()> {
        let made_changes = transform::transform_missing_chunks(source, &self.path, &self.network)?;
        if made_changes {
            manifest::generate(&self.path, &self.network)?;
        } else {
            println!(
                "No changes made to index (new chunks did not cross the {} block threshold).",
                BLOCKS_PER_VOLUME
            )
        }
        Ok(())
    }
    /// Creates a new manifest file.
    pub fn maintainer_generate_manifest(&self) -> Result<()> {
        manifest::generate(&self.path, &self.network)?;
        Ok(())
    }
    pub fn read_manifest(&self) -> Result<IndexManifest> {
        manifest::read(&self.path, &self.network)
    }
    /// Retrieves all transaction identifiers for all transactions in which the given address appears.
    pub fn find_transactions(&self, address: &str) -> Result<Vec<AppearanceTx>> {
        discover::single_address(address, &self.path, &self.network)
    }
    /// Checks local data against the contents of the manifest.
    pub fn check_completeness(&self) -> Result<IndexCompleteness> {
        manifest::completeness_audit(&self.path, &self.network)
    }
    /// TODO
    pub fn maintainer_audit_correctness(&self, compare_with: &UnchainedPath) -> Result<()> {
        todo!("Index audit against Unchained Index data not yet implemented.");
        Ok(())
    }
    /// Fetches the sample data and places it in the project data directory.
    pub async fn get_sample_data(&self, unchained: &UnchainedPath) -> Result<()> {
        fetch::samples(&self.path, unchained, &self.network).await?;
        manifest::generate(&self.path, &self.network)?;
        Ok(())
    }
}

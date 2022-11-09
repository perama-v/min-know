//! Extract transaction information about addresses from local
//! address-appearance-index files.
//!
//! Used to discover information useful for a user.
//!
//! See the index [spec][1] for more information.
//!
//! [1]: https://github.com/perama-v/address-appearance-index-specs

use std::{fs, path::PathBuf};

use anyhow::anyhow;
use anyhow::Context;

use crate::constants;
use crate::encoding;
use crate::spec::{AddressIndexVolume, AppearanceTx};
use crate::types::{AddressIndexPath, Network};
use crate::utils;
use crate::utils::hex_string_to_bytes;

/// Retrieves all transaction identifiers for all transactions in which
/// the given address appears.
///
/// # Example
/// ## Sample data
/// Retrieve address data from sample data:
/// ```
/// use min_know::{discover::single_address, types::{AddressIndexPath, Network}};
/// use std::path::PathBuf;
///
/// // An address. E.g., EF dev wallet.
/// let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
/// let data_dir = AddressIndexPath::Sample;
/// // Default network is mainnet.
/// let network = Network::default();
/// let appearances = single_address(address, &data_dir, &network)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
/// ## Real Data
/// Retrieve address data from real data:
/// ```
/// use min_know::{discover::single_address, types::{AddressIndexPath, Network}};
/// use std::path::PathBuf;
///
/// let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
/// let my_dir = PathBuf::from("./my_source");
/// let data_dir = AddressIndexPath::Sample;
/// let network = Network::default();
/// let appearances = single_address(address, &data_dir, &network)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn single_address(
    address: &str,
    source: &AddressIndexPath,
    network: &Network,
) -> Result<Vec<AppearanceTx>, anyhow::Error> {
    let address = address.trim_start_matches("0x");
    let chapter = utils::address_to_chapter(address)?;
    let chap_dir = source.chapter_dir(&network, &chapter)?;

    let files =
        fs::read_dir(&chap_dir).with_context(|| format!("Failed to read dir {:?}", chap_dir))?;
    let byte_len = hex_string_to_bytes(address)?.len() as u32;
    if byte_len != constants::DEFAULT_BYTES_PER_ADDRESS {
        return Err(anyhow!(
            "address must be {} bytes. Received {} bytes.",
            constants::DEFAULT_BYTES_PER_ADDRESS,
            byte_len
        ));
    }
    let mut res: Vec<AppearanceTx> = vec![];
    // Parse all files in the chapter.
    for filename in files {
        let path = filename?.path();
        let ssz_snappy_data =
            fs::read(&path).with_context(|| format!("Failed to read files from {:?}", path))?;
        // Decode and decompress.
        let volume_data: AddressIndexVolume = encoding::decode_and_decompress(ssz_snappy_data)?;
        // Find address and get transactions.
        let txs = volume_data
            .addresses
            .into_iter()
            .filter(|a| a.starts_with_hex(address))
            .flat_map(|a| a.appearances.into_iter());
        res.extend(txs);
    }
    // Sort by block.
    res.sort_by_key(|x| x.block);
    Ok(res)
}

/// Retrieves and decodes a single [index volume][0] .ssz_snappy file.
///
/// [0]: https://github.com/perama-v/address-appearance-index-specs#addressindexvolume
///
/// # Example
/// Gets the path for file: "chapter_0x4e_014_100_000.ssz_snappy"
/// ```rust
/// use min_know::{
///     discover::single_volume,
///     types::{AddressIndexPath, Network}};
///
/// let data_dir = AddressIndexPath::Sample;
/// let network = Network::default();
/// let chapter = "4e";
/// let volume_oldest = 14_100_000;
///
/// let volume_path = data_dir
///     .volume_file(&network, chapter, volume_oldest)?;
///
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn single_volume(path: PathBuf) -> Result<AddressIndexVolume, anyhow::Error> {
    let ssz_snappy_data =
        fs::read(&path).with_context(|| format!("Failed to read files from {:?}", &path))?;
    // Decode and decompress
    let data = encoding::decode_and_decompress(ssz_snappy_data)?;
    Ok(data)
}

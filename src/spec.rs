//! Types defined in the address-appearance-index [specification][1].
//!
//! [1]: https://github.com/perama-v/address-appearance-index-specs
use serde::{Serialize, Deserialize};
use ssz;
use ssz_derive::{Decode, Encode};
use ssz_types::{
    typenum::{U1, U1073741824, U20, U256, U32, U65536, U1024, U64},
    FixedVector, VariableList,
};
use tree_hash::Hash256;
use tree_hash_derive::TreeHash;

use crate::unchained::structure::{AddressData, TransactionId};

/// Content of an entry in the Appearances (transactions) table.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#appearancetx
#[derive(Debug, PartialEq, Clone, Copy, Encode, Decode, TreeHash)]
pub struct AppearanceTx {
    /// The Ethereum execution block number.
    pub block: u32,
    /// The index of the transaction in a block.
    pub index: u32,
}

impl AppearanceTx {
    /// Converts TransactionId into an SSZ compliant format.
    ///
    /// The Unchained Index library has a similar struct that
    /// is not SSZ compliant. This function converts into the
    /// compliant form.
    pub fn from_unchained_format(unchained: &TransactionId) -> Self {
        AppearanceTx {
            block: unchained.block,
            index: unchained.index,
        }
    }
    /// Converts to web3.rs transaction type.
    pub fn as_web3_tx_id(&self) -> web3::types::TransactionId {
        let tx_block_id =
            web3::types::BlockId::Number(web3::types::BlockNumber::Number(<_>::from(self.block)));
        let tx_index = <_>::from(self.index);
        web3::types::TransactionId::Block(tx_block_id, tx_index)
    }
}

/// Holds selected transactions for serialization and storage.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#addressappearances
#[derive(Debug, Default, PartialEq, Clone, Encode, Decode, TreeHash)]
pub struct AddressAppearances {
    /// The address that appeared in a transaction.
    ///
    /// The maximum length of the list is derived as follows:
    ///
    /// `DEFAULT_BYTES_PER_ADDRESS = 20`.
    pub address: FixedVector<u8, U20>,
    /// The transactions where the address appeared.
    ///
    /// Note that the maximum value for the VariableList is as follows.
    ///
    /// `MAX_TXS_PER_BLOCK_RANGE = 2**30 = U1073741824`.
    pub appearances: VariableList<AppearanceTx, U1073741824>,
}

impl AddressAppearances {
    /// Converts AddressData into an SSZ compliant format.
    ///
    /// The Unchained Index library has a similar struct that
    /// is not SSZ compliant. This function converts into the
    /// compliant form.
    pub fn from_unchained_format(unchained: AddressData) -> Self {
        let txs: Vec<AppearanceTx> = unchained
            .appearances
            .iter()
            .map(AppearanceTx::from_unchained_format)
            .collect();
        AddressAppearances {
            address: <_>::from(unchained.address),
            appearances: <_>::from(txs),
        }
    }
    /// Checks if the address starts with a given hex string.
    ///
    /// Hex string may be an odd number of bytes, so cannot directly compare bytes.
    pub fn starts_with_hex(&self, hex: &str) -> bool {
        hex::encode(self.address.to_vec()).starts_with(hex)
    }
}

/// Holds address appearance data for specific addresses and blocks.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#addressindexvolume
#[derive(PartialEq, Debug, Encode, Decode, Clone, TreeHash)]
pub struct AddressIndexVolume {
    /// Prefix common to all addresses that this data covers.
    ///
    /// The maximum length of the list is derived as follows:
    ///
    /// `DEFAULT_BYTES_PER_ADDRESS = 20`.
    pub address_prefix: FixedVector<u8, U20>,
    /// The blocks that this chunk data covers.
    pub identifier: VolumeIdentifier,
    /// The addresses that appeared in this range and the relevant transactions.
    ///
    /// The maximum length of the list is derived as follows:
    ///
    /// `MAX_ADDRESSES_PER_BLOCK_RANGE = 2**30 = U1073741824`.
    pub addresses: VariableList<AddressAppearances, U1073741824>,
}

/// Refers to a particular volume of the index by using the oldest possible block
/// it may contain.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#volumeidentifier
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, TreeHash, Serialize, Deserialize)]
pub struct VolumeIdentifier {
    pub oldest_block: u32,
}

/// Represents a store for the hash of a volume.
///
/// Used for constructing the index manifest.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#manifestvolume
#[derive(Debug, Decode, Encode, Clone, Serialize, Deserialize)]
pub struct ManifestVolume {
    pub identifier: VolumeIdentifier,
    pub tree_hash_root: Hash256,
}

/// Refers to a particular index chapter and defines which address are part of that
/// chapter.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#chapteridentifier
#[derive(Clone, Debug, Decode, Default, Encode, Serialize, Deserialize)]
pub struct ChapterIdentifier {
    /// The byte representation of hex characters that similar addresses share.
    ///
    /// The number of characters are defined by `ADDRESS_CHARS_SIMILARITY_DEPTH`, and the bytes needed to represent these characters are defined as `NUM_COMMON_BYTES`.
    pub address_common_bytes: FixedVector<u8, U1>,
}

impl ChapterIdentifier {
    /// Gets name of chapter directory that the chapter identifier refers to.
    pub fn as_string(&self) -> String {
        hex::encode(self.address_common_bytes.to_vec())
    }
}

/// Holds address manifest data for specific chapter.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#manifestchapter
#[derive(Debug, Decode, Default, Encode, Clone, Serialize, Deserialize)]
pub struct ManifestChapter {
    /// Used to refer to the given chapter.
    pub identifier: ChapterIdentifier,
    /// Represents the metadata of volumes within a single chapter.
    ///
    /// The maximum list length is defined as follows:
    ///
    /// `MAX_RANGES = 2**16 = U65536`.
    pub volume_metadata: VariableList<ManifestVolume, U65536>,
}

/// An SSZ list of the bytes that represent a network name string.
///
/// This is mostly an internal type, and you may be looking for
/// [`super::types::Network`] instead.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#networkname
#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct NetworkName {
    /// The network name as ASCII-encoded bytes.
    ///
    /// The maximum length of the list is defined as:
    ///
    /// `MAX_NETWORK_NAME_BYTES = 2**5 = U32`.
    pub name: VariableList<u8, U32>,
}

/// Holds the semantic version of the address-appearance-index specification.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#indexspecificationversion
#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct IndexSpecificationVersion{
    pub major: u32,
    pub minor: u32,
    pub patch: u32
}

/// Represents a link to the address-appearance-index specification.
///
/// For example: A url string or an IPFS CID string encoded in 128 bytes.
#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct IndexSpecificationSchemas {
    pub resource: VariableList<u8, U1024>
}


/// Represents an identifier that can be used to publish the index manifest under.
///
/// # Example
///
/// A smart contract that stores a topic string and manifest IPFS CID pair for lookup.
/// The topic string to be used may be ASCII-decoded `resource` bytes.
///
/// E.g., "address-appearance-index-mainnet".
#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct IndexPublishingIdentifier {
    pub topic: VariableList<u8, U64>
}


/// Represents a file containing metadata about the index.
///
/// This type is defined in the [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#indexmanifest
#[derive(Debug, Decode, Encode, Serialize, Deserialize)]
pub struct IndexManifest {
    pub version: IndexSpecificationVersion,
    pub schemas: IndexSpecificationSchemas,
    pub publish_as_topic: IndexPublishingIdentifier,
    pub network: NetworkName,
    pub latest_volume_identifier: VolumeIdentifier,
    /// Contains the hashes of the volumes in the chapter.
    ///
    /// The length of the vector is defined as:
    ///
    /// `NUM_CHAPTERS = 16**ADDRESS_CHARS_SIMILARITY_DEPTH = 256`
    pub chapter_metadata: FixedVector<ManifestChapter, U256>,
}

impl IndexManifest {
    /// Gets the network name in String form.
    pub fn network_name(&self) -> Result<String, anyhow::Error> {
        Ok(String::from_utf8(self.network.name.to_vec())?)
    }
    /// Gets the file name of the manifest, without the file suffix.
    ///
    /// # Example
    /// "manifest_v_00_01_00" (no trailing ".ssz" or ".ssz_snappy").
    pub fn file_name_no_encoding(&self) -> Result<String, anyhow::Error> {
        Ok(format!(
            "manifest_v_{:02}_{:02}_{:02}",
            self.version.major, self.version.minor, self.version.patch
        ))
    }
}

//! Address Appearance Index (AAI)

use anyhow::{bail, Result};
use ssz_rs::prelude::*;
use web3::types::{BlockId, BlockNumber, TransactionId};

use crate::{
    config::choices::DataKind,
    extraction::address_appearance_index::AAIExtractor,
    manifest::address_appearance_index::AAIManifest,
    parameters::address_appearance_index::{
        BLOCKS_PER_VOLUME, DEFAULT_BYTES_PER_ADDRESS, MAX_ADDRESSES_PER_VOLUME,
        MAX_RECORDS_PER_CHAPTER, MAX_TXS_PER_VOLUME, NUM_CHAPTERS, NUM_COMMON_BYTES,
    },
    samples::address_appearance_index::AAISampleObtainer,
    utils::{self, unchained::types::BlockRange},
};

use super::traits::*;

/// Spec for the Address Appearance Index database.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash)]
pub struct AAISpec {}

impl DataSpec for AAISpec {
    const NUM_CHAPTERS: usize = NUM_CHAPTERS as usize;

    // const MAX_VOLUMES: usize = 1_000_000_000;

    type AssociatedVolumeId = AAIVolumeId;

    type AssociatedChapterId = AAIChapterId;

    type AssociatedChapter = AAIChapter;

    type AssociatedRecord = AAIRecord;

    type AssociatedRecordKey = AAIRecordKey;

    type AssociatedRecordValue = AAIRecordValue;

    type AssociatedExtractor = AAIExtractor;

    type AssociatedSampleObtainer = AAISampleObtainer;

    type AssociatedManifest = AAIManifest;

    fn spec_matches_input(data_kind: &DataKind) -> bool {
        matches!(data_kind, DataKind::AddressAppearanceIndex(_))
    }

    fn spec_version() -> String {
        String::from("0.1.0")
    }

    fn spec_schemas_resource() -> String {
        String::from("https://github.com/perama-v/address-index/tree/main/address_appearance_index")
    }

    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> Result<Self::AssociatedChapterId> {
        let bytes = record_key.key[0..2].to_vec();
        Ok(AAIChapterId {
            val: Vector::from_iter(bytes),
        })
    }

    fn raw_key_as_record_key(key: &str) -> Result<Self::AssociatedRecordKey> {
        let raw_bytes = hex::decode(key.trim_start_matches("0x"))?;
        Ok(AAIRecordKey {
            key: Vector::from_iter(raw_bytes),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, SimpleSerialize)]
pub struct AAIVolumeId {
    pub oldest_block: u32,
}
impl VolumeIdMethods<AAISpec> for AAIVolumeId {
    fn interface_id(&self) -> String {
        format!(
            "volume_{}",
            utils::string::num_as_triplet(self.oldest_block)
        )
    }
    fn nth_id(n: u32) -> Result<Self> {
        // n=0, id=0
        // n=1, id=100_000
        // n=2, id=200_000
        let oldest_block = n * BLOCKS_PER_VOLUME;
        Ok(AAIVolumeId { oldest_block })
    }

    fn is_nth(&self) -> Result<u32> {
        // id=0, n=0
        // id=100_000, n=1
        // id=200_000, n=2
        Ok(self.oldest_block / BLOCKS_PER_VOLUME)
    }

    fn from_interface_id(interface_id: &str) -> Result<Self> {
        let oldest_block = interface_id
            .trim_start_matches("volume")
            .replace('_', "")
            .parse::<u32>()?;
        Ok(AAIVolumeId { oldest_block })
    }
}
impl AAIVolumeId {
    pub(crate) fn to_block_range(&self) -> Result<BlockRange> {
        BlockRange::new(self.oldest_block, BLOCKS_PER_VOLUME - 1 + self.oldest_block)
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct AAIChapterId {
    pub val: Vector<u8, NUM_COMMON_BYTES>,
}
impl ChapterIdMethods<AAISpec> for AAIChapterId {
    fn interface_id(&self) -> String {
        let chars = hex::encode(&self.val);
        format!("chapter_0x{}", chars)
    }
    fn nth_id(n: u32) -> Result<Self> {
        if n as usize >= AAISpec::NUM_CHAPTERS {
            bail!("'n' must be <= NUM_CHAPTERS")
        }
        let byte_vec = vec![n as u8];
        Ok(AAIChapterId {
            val: Vector::from_iter(byte_vec),
        })
    }
    fn from_interface_id(id_string: &str) -> Result<Self> {
        let string = id_string.trim_start_matches("chapter_0x");
        let bytes = hex::decode(string)?;
        Ok(AAIChapterId {
            val: Vector::from_iter(bytes),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct AAIChapter {
    pub chapter_id: AAIChapterId,
    pub volume_id: AAIVolumeId,
    pub records: List<AAIRecord, MAX_RECORDS_PER_CHAPTER>,
}
impl ChapterMethods<AAISpec> for AAIChapter {
    fn volume_id(&self) -> &AAIVolumeId {
        &self.volume_id
    }

    fn chapter_id(&self) -> &AAIChapterId {
        &self.chapter_id
    }

    fn records(&self) -> &Vec<AAIRecord> {
        &self.records
    }

    fn as_serialized_bytes(&self) -> Result<Vec<u8>> {
        Ok(serialize::<Self>(self)?)
    }
    /// Reads a Chapter from file. Currently reads Relic file structure.
    fn from_file(data: Vec<u8>) -> Result<Self> {
        // Files are ssz encoded.
        let chapter = match deserialize::<Self>(&data) {
            Ok(c) => c,
            Err(e) => bail!(
                "Could not decode the SSZ data. Check that the library
            spec version matches the version in the manifest.  {:?}",
                e
            ),
        };
        Ok(chapter)
    }
    fn filename(&self) -> String {
        format!(
            "{}_{}.ssz",
            self.volume_id.interface_id(),
            self.chapter_id.interface_id()
        )
    }

    fn new_empty(volume_id: &AAIVolumeId, chapter_id: &AAIChapterId) -> Self {
        AAIChapter {
            chapter_id: chapter_id.clone(),
            volume_id: volume_id.clone(),
            records: List::default(),
        }
    }
}

impl AAIChapter {
    /// A helper function that converts the old data chapter (file) structure into the new
    /// one.
    ///
    /// This is used to minimise changes to the transformation/exctraction code during
    /// the move to generics and can replaced eventually.
    pub(crate) fn from_relic(data: RelicChapter) -> Self {
        let chapter_id = AAIChapterId {
            val: Vector::from_iter(data.address_prefix.to_vec()),
        };
        let volume_id = AAIVolumeId {
            oldest_block: data.identifier.oldest_block,
        };
        let mut records: Vec<AAIRecord> = vec![];
        for item in data.addresses.iter() {
            let r = AAIRecord {
                key: AAIRecordKey {
                    key: Vector::from_iter(item.address.to_vec()),
                },
                value: AAIRecordValue {
                    value: List::from_iter(item.appearances.to_vec()),
                },
            };
            records.push(r)
        }
        AAIChapter {
            chapter_id,
            volume_id,
            records: List::from_iter(records),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct AAIRecord {
    pub key: AAIRecordKey,
    pub value: AAIRecordValue,
}
impl RecordMethods<AAISpec> for AAIRecord {
    fn key(&self) -> &AAIRecordKey {
        &self.key
    }

    fn value(&self) -> &AAIRecordValue {
        &self.value
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct AAIRecordKey {
    pub key: Vector<u8, DEFAULT_BYTES_PER_ADDRESS>,
}
impl RecordKeyMethods for AAIRecordKey {
    fn summary_string(&self) -> Result<String> {
        Ok(hex::encode(&self.key))
    }
}

/// Equivalent to AddressAppearances. Consists of a single address and some
/// number of transaction identfiers (appearances).
#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct AAIRecordValue {
    /// The transactions where the address appeared.
    pub value: List<AAIAppearanceTx, MAX_TXS_PER_VOLUME>,
}
impl RecordValueMethods for AAIRecordValue {
    /// Return a String representation of the contents of the RecordValue.
    fn summary_strings(&self) -> Result<Vec<String>> {
        let mut s: Vec<String> = vec![];
        for v in self.value.iter() {
            let v_str = format!("Tx in block: {}, index: {}", v.block, v.index);
            s.push(v_str)
        }
        Ok(s)
    }
}

/// An identifier for a single transaction.
///
/// Consists of block number and index within that block.
#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct AAIAppearanceTx {
    /// The Ethereum execution block number.
    pub block: u32,
    /// The index of the transaction in a block.
    pub index: u32,
}

impl AAIAppearanceTx {
    /// Converts to web3.rs transaction type.
    pub fn as_web3_tx_id(&self) -> web3::types::TransactionId {
        let block_num = BlockNumber::Number(<_>::from(self.block));
        let tx_block_id = BlockId::Number(block_num);
        let tx_index = <_>::from(self.index);
        TransactionId::Block(tx_block_id, tx_index)
    }
}

//
//
// Relic structures. The files are currently stored in this format, but this
// can be changed to a simpler format (using RecordKey and RecordValue directly).
//
//

#[derive(PartialEq, Debug, Default, SimpleSerialize, Clone)]
pub struct RelicChapter {
    /// Prefix common to all addresses that this data covers.
    pub address_prefix: Vector<u8, DEFAULT_BYTES_PER_ADDRESS>,
    /// The blocks that this chunk data covers.
    pub identifier: RelicVolumeIdentifier,
    /// The addresses that appeared in this range and the relevant transactions.
    pub addresses: List<RelicAddressAppearances, MAX_ADDRESSES_PER_VOLUME>,
}

#[derive(Debug, Default, PartialEq, Clone, SimpleSerialize)]
pub struct RelicAddressAppearances {
    /// The address that appeared in a transaction.
    pub address: Vector<u8, DEFAULT_BYTES_PER_ADDRESS>,
    /// The transactions where the address appeared.
    pub appearances: List<AAIAppearanceTx, MAX_TXS_PER_VOLUME>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, SimpleSerialize)]
pub struct RelicVolumeIdentifier {
    pub oldest_block: u32,
}

#[test]
fn encode_decode() -> Result<()> {
    use crate::specs::address_appearance_index::AAIAppearanceTx;
    let data_in = AAIAppearanceTx {
        block: 122455,
        index: 23,
    };
    let encoded = serialize(&data_in).unwrap();
    let data_out: AAIAppearanceTx = deserialize(&encoded).unwrap();
    assert_eq!(data_in, data_out);
    Ok(())
}

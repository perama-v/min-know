//! Address Appearance Index (AAI)
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};
use ssz_types::{
    typenum::{U1073741824, U20},
    FixedVector, VariableList,
};
use tree_hash_derive::TreeHash;

use crate::{
    config::dirs::DataKind,
    encoding::decode_and_decompress,
    extraction::address_appearance_index::AAIExtractor,
    parameters::address_appearance_index::{
        BLOCKS_PER_VOLUME, DEFAULT_BYTES_PER_ADDRESS, MAX_ADDRESSES_PER_VOLUME, MAX_TXS_PER_VOLUME,
        NUM_COMMON_BYTES,
    },
    samples::address_appearance_index::AAISampleObtainer,
    unchained::types::BlockRange,
};

use super::traits::*;

/// Spec for the Address Appearance Index database.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AAISpec {}

impl DataSpec for AAISpec {
    const NUM_CHAPTERS: usize = 256;

    const MAX_VOLUMES: usize = 1_000_000_000;

    type AssociatedVolumeId = AAIVolumeId;

    type AssociatedChapterId = AAIChapterId;

    type AssociatedChapter = AAIChapter;

    type AssociatedRecord = AAIRecord;

    type AssociatedRecordKey = AAIRecordKey;

    type AssociatedRecordValue = AAIRecordValue;

    type AssociatedExtractor = AAIExtractor;

    type AssociatedSampleObtainer = AAISampleObtainer;

    fn spec_name() -> SpecId {
        SpecId::AddressAppearanceIndex
    }

    fn num_chapters() -> usize {
        Self::NUM_CHAPTERS
    }

    fn record_key_to_volume_id(record_key: Self::AssociatedRecordKey) -> Self::AssociatedVolumeId {
        todo!()
    }
    /// Gets the ChapterIds relevant for a key.
    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> Result<Self::AssociatedChapterId> {
        let bytes = record_key.key[0..2].to_vec();
        Ok(AAIChapterId {
            val: <_>::from(bytes),
        })
    }

    fn record_key_matches_chapter(
        record_key: &Self::AssociatedRecordKey,
        vol: &Self::AssociatedVolumeId,
        chapter: &Self::AssociatedChapterId,
    ) -> bool {
        todo!()
    }
    // Key is a hex string. Converts it to an ssz vector.
    fn raw_key_as_record_key(key: &str) -> Result<Self::AssociatedRecordKey> {
        let raw_bytes = hex::decode(key.trim_start_matches("0x"))?;
        Ok(AAIRecordKey {
            key: <_>::from(raw_bytes),
        })
    }

    fn raw_value_as_record_value<T>(raw_data_value: T) -> Self::AssociatedRecordValue {
        todo!()
    }

    fn new_chapter() -> Self::AssociatedChapter {
        todo!()
    }
}

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
)]
pub struct AAIVolumeId {
    pub oldest_block: u32,
}
impl VolumeIdMethods<AAISpec> for AAIVolumeId {
    fn interface_id(&self) -> String {
        let mut name = format!("{:0>9}", self.oldest_block);
        for i in [6, 3] {
            name.insert(i, '_');
        }
        format!("volume_{}", name)
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
}
impl AAIVolumeId {
    pub fn to_block_range(&self) -> Result<BlockRange> {
        Ok(BlockRange::new(
            self.oldest_block,
            BLOCKS_PER_VOLUME - 1 + self.oldest_block,
        )?)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct AAIChapterId {
    pub val: FixedVector<u8, NUM_COMMON_BYTES>,
}
impl ChapterIdMethods<AAISpec> for AAIChapterId {
    fn interface_id(&self) -> String {
        let chars = hex::encode(self.val.to_vec());
        format!("chapter_0x{}", chars)
    }
    fn nth_id(n: u32) -> Result<Self> {
        if n as usize >= AAISpec::NUM_CHAPTERS {
            bail!("'n' must be <= NUM_CHAPTERS")
        }
        let byte_vec = vec![n as u8];
        let Ok(fv) = FixedVector::<u8, NUM_COMMON_BYTES>::new(byte_vec) else {
            bail!("Provided vector is too long for Fixed Vector.")
        };
        Ok(AAIChapterId { val: fv })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct AAIChapter {
    pub chapter_id: AAIChapterId,
    pub volume_id: AAIVolumeId,
    pub records: Vec<AAIRecord>,
}
impl ChapterMethods<AAISpec> for AAIChapter {
    fn get(self) -> Self {
        self
    }

    fn find_record(&self, key: AAIRecordKey) -> AAIRecord {
        todo!()
    }

    fn volume_id(&self) -> &AAIVolumeId {
        &self.volume_id
    }

    fn chapter_id(&self) -> &AAIChapterId {
        &self.chapter_id
    }

    fn records(&self) -> &Vec<AAIRecord> {
        &self.records
    }

    fn as_serialized_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
    /// Reads a Chapter from file. Currently reads Relic file structure.
    fn from_file(data: Vec<u8>) -> Result<Self> {
        // Files are ssz encoded.
        let contents: RelicChapter = decode_and_decompress(data)?;
        let volume_id = AAIVolumeId {
            oldest_block: contents.identifier.oldest_block,
        };
        let chapter_id = contents.address_prefix.to_vec()[0..3].to_vec();
        let chapter_id = AAIChapterId {
            val: <_>::from(chapter_id),
        }; // contents.address_prefix;
        let mut records = vec![];
        // TODO: Change stored file structure to avoid this conversion step.
        for a in contents.addresses.to_vec() {
            let key = AAIRecordKey { key: a.address };
            let value = AAIRecordValue {
                value: a.appearances,
            };
            let record = AAIRecord { key, value };
            records.push(record);
        }
        Ok(AAIChapter {
            chapter_id,
            volume_id,
            records,
        })
    }
}

impl AAIChapter {
    /// A helper function that converts the old data chapter (file) structure into the new
    /// one.
    ///
    /// This is used to minimise changes to the transformation/exctraction code during
    /// the move to generics and can replaced eventually.
    pub fn from_relic(data: RelicChapter) -> Self {
        let chapter_id = AAIChapterId {
            val: <_>::from(data.address_prefix.to_vec()),
        };
        let volume_id = AAIVolumeId {
            oldest_block: data.identifier.oldest_block,
        };
        let mut records: Vec<AAIRecord> = vec![];
        for item in data.addresses.to_vec() {
            let mut r = AAIRecord::default();
            r.key = AAIRecordKey {
                key: <_>::from(item.address.to_vec()),
            };
            r.value = AAIRecordValue {
                value: <_>::from(item.appearances.to_vec()),
            };
            records.push(r)
        }
        AAIChapter {
            chapter_id,
            volume_id,
            records,
        }
    }
}

pub type DefaultBytesPerAddress = U20;
pub type MaxTxsPerVolume = U1073741824;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct AAIRecord {
    pub key: AAIRecordKey,
    pub value: AAIRecordValue,
}
impl RecordMethods<AAISpec> for AAIRecord {
    fn get(&self) -> &Self {
        &self
    }

    fn new(key: AAIRecordKey, val: AAIRecordValue) -> Self {
        todo!()
    }

    fn key(&self) -> &AAIRecordKey {
        &self.key
    }

    fn values_as_strings(self) -> Vec<String> {
        self.value.as_strings()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct AAIRecordKey {
    pub key: FixedVector<u8, DefaultBytesPerAddress>,
}
impl RecordKeyMethods for AAIRecordKey {
    fn get(self) -> Self {
        self
    }
}

/// Equivalent to AddressAppearances. Consists of a single address and some
/// number of transaction identfiers (appearances).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct AAIRecordValue {
    /// The transactions where the address appeared.
    pub value: VariableList<AAIAppearanceTx, MaxTxsPerVolume>,
}
impl RecordValueMethods for AAIRecordValue {
    fn get(self) -> Self {
        self
    }
    /// Return a String representation of the contents of the RecordValue.
    fn as_strings(self) -> Vec<String> {
        let mut s: Vec<String> = vec![];
        for v in self.value.to_vec() {
            let v_str = format!("Tx in block: {}, index: {}", v.block, v.index);
            s.push(v_str)
        }
        s
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct AAIAppearanceTx {
    /// The Ethereum execution block number.
    pub block: u32,
    /// The index of the transaction in a block.
    pub index: u32,
}

//
// Relic structures. The files are currently stored in this format, but this
// can be changed to a simpler format (using RecordKey and RecordValue directly).
//
#[derive(PartialEq, Debug, Encode, Decode, Clone, TreeHash)]
pub struct RelicChapter {
    /// Prefix common to all addresses that this data covers.
    pub address_prefix: FixedVector<u8, DEFAULT_BYTES_PER_ADDRESS>,
    /// The blocks that this chunk data covers.
    pub identifier: RelicVolumeIdentifier,
    /// The addresses that appeared in this range and the relevant transactions.
    pub addresses: VariableList<RelicAddressAppearances, MAX_ADDRESSES_PER_VOLUME>,
}

#[derive(Debug, Default, PartialEq, Clone, Encode, Decode, TreeHash)]
pub struct RelicAddressAppearances {
    /// The address that appeared in a transaction.
    pub address: FixedVector<u8, DEFAULT_BYTES_PER_ADDRESS>,
    /// The transactions where the address appeared.
    pub appearances: VariableList<AAIAppearanceTx, MAX_TXS_PER_VOLUME>,
}

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, TreeHash, Serialize, Deserialize)]
pub struct RelicVolumeIdentifier {
    pub oldest_block: u32,
}

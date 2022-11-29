use std::fmt::Display;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{
    length::Fixed,
    typenum::{U1073741824, U2, U20},
    FixedVector, VariableList,
};
use tree_hash_derive::TreeHash;

use crate::{
    encoding::decode_and_decompress,
    parameters::address_appearance_index::{
        DEFAULT_BYTES_PER_ADDRESS, MAX_ADDRESSES_PER_VOLUME, MAX_TXS_PER_VOLUME, NUM_COMMON_BYTES,
    },
    spec::VolumeIdentifier,
};

use super::types::*;

/// Spec for the Address Appearance Index database.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AdApInSpec {}

impl DataSpec for AdApInSpec {
    const DATABASE_INTERFACE_ID: &'static str = "address_appearance_index";

    const NUM_CHAPTERS: usize = 256;

    const MAX_VOLUMES: usize = 1_000_000_000;

    type AssociatedVolumeId = VolumeId;

    type AssociatedChapterId = ChapterId;

    type AssociatedChapter = Chapter;

    type AssociatedRecord = Record;

    type AssociatedRecordKey = RecordKey;

    type AssociatedRecordValue = RecordValue;

    fn spec_name() -> SpecId {
        SpecId::AddressAppearanceIndex
    }

    fn num_chapters() -> usize {
        Self::NUM_CHAPTERS
    }

    fn volume_interface_id<T>(volume: T) -> String {
        todo!()
    }

    fn chapter_interface_id<T>(chapter: T) -> String {
        todo!()
        // format!("chapter_{:?}", chapter)
    }

    fn get_all_chapter_ids() -> Vec<Self::AssociatedChapterId> {
        todo!()
    }

    fn get_all_volume_ids() -> Vec<Self::AssociatedVolumeId> {
        todo!()
    }

    fn record_key_to_volume_id(record_key: Self::AssociatedRecordKey) -> Self::AssociatedVolumeId {
        todo!()
    }
    /// Gets the ChapterIds relevant for a key.
    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> Result<Self::AssociatedChapterId> {
        let bytes = record_key.key[0..2].to_vec();
        Ok(ChapterId {
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
        Ok(RecordKey {
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
pub struct VolumeId {
    oldest_block: u32,
}
impl VolumeIdMethods for VolumeId {}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct ChapterId {
    val: FixedVector<u8, NUM_COMMON_BYTES>,
}
impl ChapterIdMethods for ChapterId {
    fn interface_id(&self) -> String {
        hex::encode(self.val.to_vec())
    }
    fn dir_name(&self) -> String {
        format!("chapter_0x{}", self.interface_id())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: ChapterId,
    pub volume_id: VolumeId,
    pub records: Vec<Record>,
}
impl<T> ChapterMethods<T> for Chapter
where
    T: DataSpec,
{
    type RecordType = Record;

    fn get(self) -> Self {
        self
    }

    fn find_record(&self, key: T::AssociatedRecordKey) -> T::AssociatedRecord {
        todo!()
    }

    fn volume_id(&self) -> T::AssociatedVolumeId {
        todo!()
    }

    fn chapter_id(&self) -> T::AssociatedChapterId {
        todo!()
    }

    fn records(self) -> Vec<Self::RecordType> {
        self.records
    }

    fn as_serialized_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_file(data: Vec<u8>) -> Result<Self> {
        // Files are ssz encoded.
        let contents: RelicFileStructure = decode_and_decompress(data)?;
        let volume_id = VolumeId {
            oldest_block: contents.identifier.oldest_block,
        };
        let chapter_id = contents.address_prefix.to_vec()[0..3].to_vec();
        let chapter_id = ChapterId {
            val: <_>::from(chapter_id),
        }; // contents.address_prefix;
        let mut records = vec![];
        // TODO: Change stored file structure to avoid this conversion step.
        for a in contents.addresses.to_vec() {
            let key = RecordKey { key: a.address };
            let value = RecordValue {
                value: a.appearances,
            };
            let record = Record { key, value };
            records.push(record);
        }
        Ok(Chapter {
            chapter_id,
            volume_id,
            records,
        })
    }

}

pub type DefaultBytesPerAddress = U20;
pub type MaxTxsPerVolume = U1073741824;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct Record {
    pub key: RecordKey,
    pub value: RecordValue,
}
impl RecordMethods for Record {
    fn get(&self) -> &Self {
        &self
    }

    fn new<T: DataSpec>(
        key: T::AssociatedRecordKey,
        val: T::AssociatedRecordValue,
    ) -> T::AssociatedRecord {
        todo!()
    }

    fn key<T: DataSpec>(&self) -> T::AssociatedRecordKey {
        todo!()
    }

    fn values_as_strings(self) -> Vec<String> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct RecordKey {
    pub key: FixedVector<u8, DefaultBytesPerAddress>,
}
impl RecordKeyMethods for RecordKey {
    fn get(self) -> Self {
        self
    }
}

/// Equivalent to AddressAppearances. Consists of a single address and some
/// number of transaction identfiers (appearances).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
pub struct RecordValue {
    /// The transactions where the address appeared.
    pub value: VariableList<AppearanceTx, MaxTxsPerVolume>,
}
impl RecordValueMethods for RecordValue {
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
pub struct AppearanceTx {
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
pub struct RelicFileStructure {
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
    pub appearances: VariableList<AppearanceTx, MAX_TXS_PER_VOLUME>,
}

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, TreeHash, Serialize, Deserialize)]
pub struct RelicVolumeIdentifier {
    pub oldest_block: u32,
}

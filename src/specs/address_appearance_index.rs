use std::fmt::Display;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use ssz_types::{
    typenum::{U1073741824, U2, U20},
    FixedVector, VariableList,
};
use tree_hash_derive::TreeHash;

use super::types::*;

/// Spec for the Address Appearance Index database.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AdApInSpec {}

impl DataSpec for AdApInSpec {
    const DATABASE_INTERFACE_ID: &'static str = "address_appearance_index";

    const NUM_CHAPTERS: usize = 256;

    const MAX_VOLUMES: usize = 1_000_000_000;

    type AssociatedVolumeId = VolId;

    type AssociatedChapterId = ChapterId;

    type AssociatedChapter = Chapter;

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
        record_key: Self::AssociatedRecordKey,
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
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct VolId {}
impl VolumeIdMethods for VolId {}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChapterId {
    val: FixedVector<u8, U2>,
}
impl ChapterIdMethods for ChapterId {
    fn interface_id(&self) -> String {
        hex::encode(self.val.to_vec())
    }
    fn dir_name(&self) -> String {
        format!("Chapter_0x{}", self.interface_id())
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Chapter {}
impl ChapterMethods for Chapter {}

pub type DefaultBytesPerAddress = U20;
pub type MaxTxsPerVolume = U1073741824;

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

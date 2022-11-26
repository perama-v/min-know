use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ssz_types::{
    typenum::{U1073741824, U2, U20},
    FixedVector, VariableList,
};

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
        let bytes = record_key[0..2].to_vec();
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
        match RecordKey::new(raw_bytes) {
            Ok(q) => Ok(q),
            Err(e) => Err(anyhow!(
                "could not turn record_key bytes into ssz vector {:?}",
                e
            )),
        }
    }

    fn raw_value_as_record_value<T>(raw_data_value: T) -> Self::AssociatedRecordValue {
        todo!()
    }
}
//#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
//pub struct RecordKey {}
pub type RecordKey = FixedVector<u8, DefaultBytesPerAddress>;
impl RecordKeyMethods for RecordKey {}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct VolId {}
impl VolumeIdMethods for VolId {}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChapterId {
    val: FixedVector<u8, U2>,
}
impl ChapterIdMethods for ChapterId {}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Chapter {}
impl ChapterMethods for Chapter {}

pub type DefaultBytesPerAddress = U20;
pub type MaxTxsPerVolume = U1073741824;

/// Equivalent to AddressAppearances. Consists of a single address and some
/// number of transaction identfiers (appearances).
//#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordValue {
    /// The address that appeared in a transaction.
    pub record_key: RecordKey,
    /// The transactions where the address appeared.
    pub value: VariableList<AppearanceTx, MaxTxsPerVolume>,
}
impl RecordValueMethods for RecordValue {}

//#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppearanceTx {
    /// The Ethereum execution block number.
    pub block: u32,
    /// The index of the transaction in a block.
    pub index: u32,
}

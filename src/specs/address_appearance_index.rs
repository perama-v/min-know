use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use ssz_derive::{Encode, Decode};
use ssz_types::{FixedVector, typenum::{U1073741824, U20}, VariableList};
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

    type AssociatedChapterId = ChapId;

    type AssociatedUnit = BaseUnit;

    type AssociatedQuery = Query;

    type AssociatedElement = Element;

    fn spec_name() -> SpecId {
        todo!()
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

    fn query_to_volume_id(query: Self::AssociatedQuery) -> Self::AssociatedVolumeId {
        todo!()
    }

    fn query_to_chapter_id(query: Self::AssociatedQuery) -> Self::AssociatedChapterId {
        todo!()
    }

    fn query_matches_unit(
        query: &Self::AssociatedQuery,
        vol: &Self::AssociatedVolumeId,
        chapter: &Self::AssociatedChapterId,
    ) -> bool {
        todo!()
    }
    // Key is a hex string. Converts it to an ssz vector.
    fn raw_key_as_query(key: &str) -> Result<Self::AssociatedQuery, anyhow::Error>
      {
        let raw_bytes = hex::decode(key)?;
        match Query::new(raw_bytes) {
            Ok(q) => Ok(q),
            Err(e) => Err(anyhow!("could not turn query bytes into ssz vector {:?}", e))
        }
    }

    fn raw_value_as_element<T>(raw_data_value: T) -> Self::AssociatedElement {
        todo!()
    }
}
//#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
//pub struct Query {}
pub type Query = FixedVector<u8, DefaultBytesPerAddress>;
impl QueryMethods for Query {}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct VolId {}
impl VolumeIdMethods for VolId {}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ChapId {}
impl ChapterIdMethods for ChapId {}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct BaseUnit {}
impl UnitMethods for BaseUnit {}


pub type DefaultBytesPerAddress = U20;
pub type MaxTxsPerVolume = U1073741824;

/// Equivalent to AddressAppearances. Consists of a single address and some
/// number of transaction identfiers (appearances).
//#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// The address that appeared in a transaction.
    pub query: Query,
    /// The transactions where the address appeared.
    pub value: VariableList<AppearanceTx, MaxTxsPerVolume>,
}
impl ElementMethods for Element {}

//#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode, TreeHash)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AppearanceTx {
    /// The Ethereum execution block number.
    pub block: u32,
    /// The index of the transaction in a block.
    pub index: u32,
}
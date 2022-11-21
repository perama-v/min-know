use serde::{Deserialize, Serialize};

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

    fn raw_key_as_query<T>(raw_data_key: T) -> Self::AssociatedQuery {
        todo!()
    }

    fn raw_value_as_element<T>(raw_data_value: T) -> Self::AssociatedElement {
        todo!()
    }
}
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Query {}
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

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Element {}
impl ElementMethods for Element {}

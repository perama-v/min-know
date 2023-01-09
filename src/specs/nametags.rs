use std::{fs, path::Path};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    config::choices::DataKind,
    extraction::{traits::ExtractorMethods, nametags::NameTagsExtractor},
    parameters::nametags::ENTRIES_PER_VOLUME,
    samples::{nametags::SAMPLE_FILENAMES, traits::SampleObtainerMethods},
};

use super::traits::*;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsSpec {}

// Uncomment the line below to start adding a new database to this library.
impl DataSpec for NameTagsSpec {
    const NUM_CHAPTERS: usize = 256;

    type AssociatedChapter = NameTagsChapter;

    type AssociatedChapterId = NameTagsChapterId;

    type AssociatedVolumeId = NameTagsVolumeId;

    type AssociatedRecord = NameTagsRecord;

    type AssociatedRecordKey = NameTagsRecordKey;

    type AssociatedRecordValue = NameTagsRecordValue;

    type AssociatedExtractor = NameTagsExtractor;

    type AssociatedSampleObtainer = NameTagsSampleObtainer;

    type AssociatedManifest = NameTagsManifest;

    fn spec_matches_input(data_kind: &DataKind) -> bool {
        match data_kind {
            DataKind::NameTags => true,
            _ => false,
        }
    }

    fn spec_version() -> String {
        todo!()
    }

    fn spec_schemas_resource() -> String {
        todo!()
    }

    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> Result<Self::AssociatedChapterId> {
        todo!()
    }

    fn raw_key_as_record_key(key: &str) -> Result<Self::AssociatedRecordKey> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsChapter;

impl ChapterMethods<NameTagsSpec> for NameTagsChapter {
    fn get(self) -> Self {
        todo!()
    }

    fn volume_id(&self) -> &NameTagsVolumeId {
        todo!()
    }

    fn chapter_id(&self) -> &NameTagsChapterId {
        todo!()
    }

    fn records(&self) -> &Vec<NameTagsRecord> {
        todo!()
    }

    fn as_serialized_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_file(data: Vec<u8>) -> Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn filename(&self) -> String {
        todo!()
    }

    fn new_empty(volume_id: &NameTagsVolumeId, chapter_id: &NameTagsChapterId) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsChapterId;

impl ChapterIdMethods<NameTagsSpec> for NameTagsChapterId {
    fn from_interface_id(id_string: &str) -> Result<Self> {
        todo!()
    }

    fn interface_id(&self) -> String {
        todo!()
    }

    fn nth_id(n: u32) -> Result<NameTagsChapterId> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Hash, PartialOrd)]
pub struct NameTagsVolumeId {
    /// Refers to the first address in the Volume. It is index of the address
    /// where all volumes are ordered oldest to youngest.
    ///
    /// ## Example
    ///
    /// The first address in the first volume is 0, the first address in the
    /// second volume is 10000 (ENTRIES_PER_VOLUME).
    pub first_address: u32,
}

impl VolumeIdMethods<NameTagsSpec> for NameTagsVolumeId {
    fn from_interface_id(interface_id: &str) -> Result<Self> {
        todo!()
    }

    fn interface_id(&self) -> String {
        todo!()
    }

    fn nth_id(n: u32) -> Result<NameTagsVolumeId> {
        todo!()
    }

    fn is_nth(&self) -> Result<u32> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsRecord;

impl RecordMethods<NameTagsSpec> for NameTagsRecord {
    fn get(&self) -> &Self {
        todo!()
    }

    fn key(&self) -> &NameTagsRecordKey {
        todo!()
    }

    fn value(&self) -> &NameTagsRecordValue {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsRecordKey;

impl RecordKeyMethods for NameTagsRecordKey {
    fn get(self) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsRecordValue;

impl RecordValueMethods for NameTagsRecordValue {
    fn get(self) -> Self {
        todo!()
    }

    fn as_strings(&self) -> Vec<String> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsSampleObtainer;

impl SampleObtainerMethods for NameTagsSampleObtainer {
    fn raw_sample_filenames() -> Vec<&'static str> {
        SAMPLE_FILENAMES.to_vec()
    }

    fn sample_volumes() -> Option<Vec<&'static str>> {
        todo!()
    }

    fn get_raw_samples(dir: &Path) -> Result<()> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsManifest;

impl ManifestMethods<NameTagsSpec> for NameTagsManifest {
    fn spec_version(&self) -> &str {
        todo!()
    }

    fn set_spec_version(&mut self, version: String) {
        todo!()
    }

    fn schemas(&self) -> &str {
        todo!()
    }

    fn set_schemas(&mut self, schemas: String) {
        todo!()
    }

    fn database_interface_id(&self) -> &str {
        todo!()
    }

    fn set_database_interface_id(&mut self, id: String) {
        todo!()
    }

    fn latest_volume_identifier(&self) -> &str {
        todo!()
    }

    fn set_latest_volume_identifier(&mut self, volume_interface_id: String) {
        todo!()
    }

    fn cids(&self) -> Result<Vec<ManifestCids<NameTagsSpec>>> {
        todo!()
    }

    fn set_cids<U: AsRef<str> + std::fmt::Display>(
        &mut self,
        cids: &[(U, NameTagsVolumeId, NameTagsChapterId)],
    ) {
        todo!()
    }
}

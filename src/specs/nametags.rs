use serde::{Deserialize, Serialize};

use crate::{extraction::traits::ExtractorMethods, samples::{traits::SampleObtainerMethods, nametags::SAMPLE_FILENAMES}};

use super::traits::*;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsSpec {}

// Uncomment the line below to start adding a new database to this library.
impl DataSpec for NameTagsSpec {
    const NUM_CHAPTERS: usize = 256;

    type AssociatedChapter = NameTagChapter;

    type AssociatedChapterId = NameTagChapterId;

    type AssociatedVolumeId = NameTagVolumeId;

    type AssociatedRecord = NameTagRecord;

    type AssociatedRecordKey = NameTagRecordKey;

    type AssociatedRecordValue = NameTagRecordValue;

    type AssociatedExtractor = NameTagExtractor;

    type AssociatedSampleObtainer = NameTagSampleObtainer;

    type AssociatedManifest = NameTagManifest;

    fn spec_name() -> super::traits::SpecId {
        todo!()
    }

    fn spec_version() -> String {
        todo!()
    }

    fn spec_schemas_resource() -> String {
        todo!()
    }

    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> anyhow::Result<Self::AssociatedChapterId> {
        todo!()
    }

    fn raw_key_as_record_key(key: &str) -> anyhow::Result<Self::AssociatedRecordKey> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagChapter;

impl ChapterMethods<NameTagsSpec> for NameTagChapter {
    fn get(self) -> Self {
        todo!()
    }

    fn volume_id(&self) -> &NameTagVolumeId {
        todo!()
    }

    fn chapter_id(&self) -> &NameTagChapterId {
        todo!()
    }

    fn records(&self) -> &Vec<NameTagRecord> {
        todo!()
    }

    fn as_serialized_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_file(data: Vec<u8>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn filename(&self) -> String {
        todo!()
    }

    fn new_empty(volume_id: &NameTagVolumeId, chapter_id: &NameTagChapterId) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagChapterId;

impl ChapterIdMethods<NameTagsSpec> for NameTagChapterId {
    fn from_interface_id(id_string: &str) -> anyhow::Result<Self> {
        todo!()
    }

    fn interface_id(&self) -> String {
        todo!()
    }

    fn nth_id(n: u32) -> anyhow::Result<NameTagChapterId> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Hash, PartialOrd)]
pub struct NameTagVolumeId;

impl VolumeIdMethods<NameTagsSpec> for NameTagVolumeId {
    fn from_interface_id(interface_id: &str) -> anyhow::Result<Self> {
        todo!()
    }

    fn interface_id(&self) -> String {
        todo!()
    }

    fn nth_id(n: u32) -> anyhow::Result<NameTagVolumeId> {
        todo!()
    }

    fn is_nth(&self) -> anyhow::Result<u32> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagRecord;

impl RecordMethods<NameTagsSpec> for NameTagRecord {
    fn get(&self) -> &Self {
        todo!()
    }

    fn key(&self) -> &NameTagRecordKey {
        todo!()
    }

    fn value(&self) -> &NameTagRecordValue {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagRecordKey;

impl RecordKeyMethods for NameTagRecordKey {
    fn get(self) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagRecordValue;

impl RecordValueMethods for NameTagRecordValue {
    fn get(self) -> Self {
        todo!()
    }

    fn as_strings(&self) -> Vec<String> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagExtractor;

impl ExtractorMethods<NameTagsSpec> for NameTagExtractor {
    fn chapter_from_raw(
        chapter_id: &NameTagChapterId,
        volume_id: &NameTagVolumeId,
        source_dir: &std::path::Path,
    ) -> anyhow::Result<Option<NameTagChapter>> {
        todo!()
    }

    fn latest_possible_volume(source_dir: &std::path::Path) -> anyhow::Result<NameTagVolumeId> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagSampleObtainer;

impl SampleObtainerMethods for NameTagSampleObtainer {
    fn raw_sample_filenames() -> Vec<&'static str> {
        SAMPLE_FILENAMES.to_vec()
    }

    fn sample_volumes() -> Option<Vec<&'static str>> {
        todo!()
    }

    fn get_raw_samples(dir: &std::path::Path) -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagManifest;

impl ManifestMethods<NameTagsSpec> for NameTagManifest {
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

    fn cids(&self) -> anyhow::Result<Vec<ManifestCids<NameTagsSpec>>> {
        todo!()
    }

    fn set_cids<U: AsRef<str> + std::fmt::Display>(
        &mut self,
        cids: &[(U, NameTagVolumeId, NameTagChapterId)],
    ) {
        todo!()
    }
}

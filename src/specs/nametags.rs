use std::{fs, path::Path};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use ssz::{Encode, Decode};
use ssz_derive::{Encode, Decode};
use ssz_types::{FixedVector, VariableList};

use crate::{
    config::choices::DataKind,
    extraction::{
        nametags::{NameTagsExtractor, RawValue},
        traits::ExtractorMethods,
    },
    parameters::nametags::{
        BytesForAddressChars, BytesPerAddress, MaxBytesPerName, MaxBytesPerTag, MaxNamesPerRecord,
        MaxTagsPerRecord, ENTRIES_PER_VOLUME,
    },
    samples::{nametags::SAMPLE_FILENAMES, traits::SampleObtainerMethods},
    utils,
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct NameTagsChapter {
    pub chapter_id: NameTagsChapterId,
    pub volume_id: NameTagsVolumeId,
    pub records: Vec<NameTagsRecord>,
}

impl ChapterMethods<NameTagsSpec> for NameTagsChapter {
    fn volume_id(&self) -> &NameTagsVolumeId {
        &self.volume_id
    }

    fn chapter_id(&self) -> &NameTagsChapterId {
        &self.chapter_id
    }

    fn records(&self) -> &Vec<NameTagsRecord> {
        &self.records
    }

    fn as_serialized_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct NameTagsChapterId {
    pub val: FixedVector<u8, BytesForAddressChars>,
}

impl ChapterIdMethods<NameTagsSpec> for NameTagsChapterId {
    fn from_interface_id(id_string: &str) -> Result<Self> {
        let string = id_string.trim_start_matches("addresses_0x");
        let bytes = hex::decode(string)?;
        Ok(NameTagsChapterId {
            val: <_>::from(bytes),
        })
    }

    fn interface_id(&self) -> String {
        format!("addresses_0x{}", self.as_str())
    }

    fn nth_id(n: u32) -> Result<NameTagsChapterId> {
        if n as usize >= NameTagsSpec::NUM_CHAPTERS {
            bail!("'n' must be <= NUM_CHAPTERS")
        }
        let byte_vec = vec![n as u8];
        let Ok(fv) = FixedVector::<u8, BytesForAddressChars>::new(byte_vec) else {
            bail!("Provided vector is too long for Fixed Vector.")
        };
        Ok(NameTagsChapterId { val: fv })
    }
}

impl NameTagsChapterId {
    /// Returns the ChapterId as a hex string (no 0x prefix).
    pub fn as_str(&self) -> String {
        hex::encode(self.val.to_vec())
    }
    /// Determines if leading string matches the Chapter.
    pub fn matches(&self, leading: &str) -> bool {
        let s = self.as_str();
        s.starts_with(&leading)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Hash, PartialOrd, Encode, Decode)]
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
        // From the spec: "nametags_from_000_630_000"
        format!(
            "nametags_from_{}",
            utils::string::num_as_triplet(self.first_address)
        )
    }

    fn nth_id(n: u32) -> Result<NameTagsVolumeId> {
        Ok(NameTagsVolumeId {
            first_address: n * ENTRIES_PER_VOLUME,
        })
    }

    fn is_nth(&self) -> Result<u32> {
        Ok(self.first_address / ENTRIES_PER_VOLUME)
    }
}

impl NameTagsVolumeId {
    /// Determines if a globally-indexed entry is present in a volume.
    pub fn contains_entry(&self, index: u32) -> bool {
        index >= self.first_address && index < (self.first_address + ENTRIES_PER_VOLUME)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct NameTagsRecord {
    pub record_key: NameTagsRecordKey,
    pub record_value: NameTagsRecordValue,
}

impl RecordMethods<NameTagsSpec> for NameTagsRecord {

    fn key(&self) -> &NameTagsRecordKey {
        todo!()
    }

    fn value(&self) -> &NameTagsRecordValue {
        todo!()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct NameTagsRecordKey {
    key: FixedVector<u8, BytesPerAddress>,
}

impl RecordKeyMethods for NameTagsRecordKey {}

impl NameTagsRecordKey {
    pub fn from_address(address: &str) -> Result<Self> {
        let raw_bytes = hex::decode(address.trim_start_matches("0x"))?;
        Ok(NameTagsRecordKey {
            key: <_>::from(raw_bytes),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct NameTagsRecordValue {
    pub names: VariableList<Name, MaxNamesPerRecord>,
    pub tags: VariableList<Tag, MaxTagsPerRecord>,
}

impl NameTagsRecordValue {
    pub fn from_strings(names: Vec<String>, tags: Vec<String>) -> Self {
        let mut name_vec = vec![];
        for n in names {
            name_vec.push(Name::from_string(&n))
        }
        let mut tag_vec = vec![];
        for t in tags {
            tag_vec.push(Tag::from_string(&t))
        }
        NameTagsRecordValue {
            names: <_>::from(name_vec),
            tags: <_>::from(tag_vec),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct Name {
    pub val: VariableList<u8, MaxBytesPerName>,
}

impl Name {
    pub fn from_string(s: &str) -> Self {
        Name {
            val: <_>::from(s.as_bytes().to_vec()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct Tag {
    pub val: VariableList<u8, MaxBytesPerTag>,
}

impl Tag {
    pub fn from_string(s: &str) -> Self {
        Tag {
            val: <_>::from(s.as_bytes().to_vec()),
        }
    }
}

impl RecordValueMethods for NameTagsRecordValue {
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

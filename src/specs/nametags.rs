use std::str::from_utf8;

use anyhow::{bail, Result};
use ssz_rs::prelude::*;

use crate::manifest::nametags::NameTagsManifest;
use crate::parameters::nametags::MAX_RECORDS_PER_CHAPTER;
use crate::{
    config::choices::DataKind,
    extraction::nametags::NameTagsExtractor,
    parameters::nametags::{
        BYTES_FOR_ADDRESS_CHARS, BYTES_PER_ADDRESS, ENTRIES_PER_VOLUME, MAX_BYTES_PER_NAME,
        MAX_BYTES_PER_TAG, MAX_NAMES_PER_RECORD, MAX_TAGS_PER_RECORD,
    },
    samples::nametags::NameTagsSampleObtainer,
    utils,
};

use super::traits::*;

#[derive(Clone, Debug, Default, PartialEq)]
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
        matches!(data_kind, DataKind::NameTags)
    }

    fn spec_version() -> String {
        String::from("0.1.0")
    }

    fn spec_schemas_resource() -> String {
        String::from("https://github.com/perama-v/TODD/blob/main/example_specs/nametag.md")
    }

    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> Result<Self::AssociatedChapterId> {
        let bytes = record_key.key[0..2].to_vec();
        Ok(NameTagsChapterId {
            val: Vector::from_iter(bytes),
        })
    }

    fn raw_key_as_record_key(key: &str) -> Result<Self::AssociatedRecordKey> {
        let raw_bytes = hex::decode(key.trim_start_matches("0x"))?;
        Ok(NameTagsRecordKey {
            key: Vector::from_iter(raw_bytes),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct NameTagsChapter {
    pub chapter_id: NameTagsChapterId,
    pub volume_id: NameTagsVolumeId,
    pub records: List<NameTagsRecord, MAX_RECORDS_PER_CHAPTER>,
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

    fn as_serialized_bytes(&self) -> Result<Vec<u8>> {
        Ok(serialize::<Self>(self)?)
    }

    fn from_file(data: Vec<u8>) -> Result<Self>
    where
        Self: Sized,
    {
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

    fn new_empty(volume_id: &NameTagsVolumeId, chapter_id: &NameTagsChapterId) -> Self {
        NameTagsChapter {
            chapter_id: chapter_id.clone(),
            volume_id: volume_id.clone(),
            records: List::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct NameTagsChapterId {
    pub val: Vector<u8, BYTES_FOR_ADDRESS_CHARS>,
}

impl ChapterIdMethods<NameTagsSpec> for NameTagsChapterId {
    fn from_interface_id(id_string: &str) -> Result<Self> {
        let string = id_string.trim_start_matches("addresses_0x");
        let bytes = hex::decode(string)?;
        Ok(NameTagsChapterId {
            val: Vector::from_iter(bytes),
        })
    }

    fn interface_id(&self) -> String {
        format!("addresses_0x{}", self.as_string())
    }

    fn nth_id(n: u32) -> Result<NameTagsChapterId> {
        if n as usize >= NameTagsSpec::NUM_CHAPTERS {
            bail!("'n' must be <= NUM_CHAPTERS")
        }
        let byte_vec = vec![n as u8];
        Ok(NameTagsChapterId {
            val: Vector::from_iter(byte_vec),
        })
    }
}

impl NameTagsChapterId {
    /// Returns the ChapterId as a hex string (no 0x prefix).
    pub fn as_string(&self) -> String {
        hex::encode(&self.val)
    }
    /// Returns true if the candidate string starts with the ChapterId.
    pub fn matches(&self, candidate: &str) -> bool {
        candidate.starts_with(&self.as_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Hash, PartialOrd, SimpleSerialize)]
pub struct NameTagsVolumeId {
    /// Refers to the first address in the Volume. It is index of the address
    /// where all volumes are ordered oldest to youngest.
    ///
    /// ## Example
    ///
    /// The first address in the first volume is 0, the first address in the
    /// second volume is 1000 (ENTRIES_PER_VOLUME).
    pub first_address: u32,
}

impl VolumeIdMethods<NameTagsSpec> for NameTagsVolumeId {
    fn from_interface_id(interface_id: &str) -> Result<Self> {
        let Ok(first_address) = interface_id
            .trim_start_matches("nametags_from_")
            .replace('_', "")
            .parse::<u32>()
            else {
                bail!("The string: {} was not formatted as expected.", interface_id)};

        Ok(NameTagsVolumeId { first_address })
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
            first_address: n * ENTRIES_PER_VOLUME as u32,
        })
    }

    fn is_nth(&self) -> Result<u32> {
        Ok(self.first_address / ENTRIES_PER_VOLUME as u32)
    }
}

impl NameTagsVolumeId {
    /// Determines if a globally-indexed entry is present in a volume.
    pub fn contains_entry(&self, index: u32) -> bool {
        index >= self.first_address && index < (self.first_address + ENTRIES_PER_VOLUME as u32)
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct NameTagsRecord {
    pub key: NameTagsRecordKey,
    pub value: NameTagsRecordValue,
}

impl RecordMethods<NameTagsSpec> for NameTagsRecord {
    fn key(&self) -> &NameTagsRecordKey {
        &self.key
    }

    fn value(&self) -> &NameTagsRecordValue {
        &self.value
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct NameTagsRecordKey {
    key: Vector<u8, BYTES_PER_ADDRESS>,
}

impl RecordKeyMethods for NameTagsRecordKey {
    fn summary_string(&self) -> Result<String> {
        Ok(hex::encode(&self.key))
    }
}

impl NameTagsRecordKey {
    pub fn from_address(address: &str) -> Result<Self> {
        let raw_bytes = hex::decode(address.trim_start_matches("0x"))?;
        Ok(NameTagsRecordKey {
            key: Vector::from_iter(raw_bytes),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct NameTagsRecordValue {
    pub names: List<Name, MAX_NAMES_PER_RECORD>,
    pub tags: List<Tag, MAX_TAGS_PER_RECORD>,
}

impl RecordValueMethods for NameTagsRecordValue {
    fn summary_strings(&self) -> Result<Vec<String>> {
        let n = format!("names: {:?}", self.names_as_strings()?);
        let t = format!("tags: {:?}", self.tags_as_strings()?);
        Ok(vec![n, t])
    }
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
            names: List::from_iter(name_vec),
            tags: List::from_iter(tag_vec),
        }
    }
    /// Turns SSZ bytes into a vector of readable strings.
    pub fn names_as_strings(&self) -> Result<Vec<String>> {
        let mut s = vec![];
        for n in &self.names.to_vec() {
            s.push(n.to_utf8_string()?)
        }
        Ok(s)
    }
    /// Turns SSZ bytes into a vector of readable strings.
    pub fn tags_as_strings(&self) -> Result<Vec<String>> {
        let mut s = vec![];
        for t in &self.tags.to_vec() {
            s.push(t.to_utf8_string()?)
        }
        Ok(s)
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct Name {
    pub val: List<u8, MAX_BYTES_PER_NAME>,
}

impl Name {
    pub fn from_string(s: &str) -> Self {
        Name {
            val: List::from_iter(s.as_bytes().to_vec()),
        }
    }
    pub fn to_utf8_string(&self) -> Result<String> {
        let v = self.val.to_vec();
        let s = from_utf8(&v)?;
        Ok(s.to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct Tag {
    pub val: List<u8, MAX_BYTES_PER_TAG>,
}

impl Tag {
    pub fn from_string(s: &str) -> Self {
        Tag {
            val: List::from_iter(s.as_bytes().to_vec()),
        }
    }
    pub fn to_utf8_string(&self) -> Result<String> {
        let v = self.val.to_vec();
        let s = from_utf8(&v)?;
        Ok(s.to_string())
    }
}

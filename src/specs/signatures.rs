use std::{fmt::Display, str::from_utf8};

use anyhow::{bail, Result};
use ssz_rs::prelude::*;

use crate::{
    config::choices::DataKind,
    extraction::signatures::SignaturesExtractor,
    manifest::signatures::SignaturesManifest,
    parameters::signatures::{
        BYTES_FOR_SIGNATURE_CHARS, BYTES_PER_SIGNATURE, MAX_BYTES_PER_TEXT,
        MAX_RECORDS_PER_CHAPTER, MAX_TEXTS_PER_RECORD, SIGNATURES_PER_VOLUME,
    },
    samples::signatures::SignaturesSampleObtainer,
    utils,
};

use super::traits::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignaturesSpec {}

impl DataSpec for SignaturesSpec {
    const NUM_CHAPTERS: usize = 256;

    type AssociatedChapter = SignaturesChapter;

    type AssociatedChapterId = SignaturesChapterId;

    type AssociatedVolumeId = SignaturesVolumeId;

    type AssociatedRecord = SignaturesRecord;

    type AssociatedRecordKey = SignaturesRecordKey;

    type AssociatedRecordValue = SignaturesRecordValue;

    type AssociatedExtractor = SignaturesExtractor;

    type AssociatedSampleObtainer = SignaturesSampleObtainer;

    type AssociatedManifest = SignaturesManifest;

    fn spec_matches_input(data_kind: &DataKind) -> bool {
        matches!(data_kind, DataKind::Signatures)
    }

    fn spec_version() -> String {
        String::from("0.1.0")
    }

    fn spec_schemas_resource() -> String {
        String::from("https://github.com/perama-v/TODD/blob/main/example_specs/signatures.md")
    }

    fn record_key_to_chapter_id(record_key: &SignaturesRecordKey) -> Result<SignaturesChapterId> {
        let bytes = record_key.key[0..2].to_vec();
        Ok(SignaturesChapterId {
            val: Vector::from_iter(bytes),
        })
    }

    fn raw_key_as_record_key(key: &str) -> Result<SignaturesRecordKey> {
        let raw_bytes = hex::decode(key.trim_start_matches("0x"))?;
        Ok(SignaturesRecordKey {
            key: Vector::from_iter(raw_bytes),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct SignaturesChapter {
    pub chapter_id: SignaturesChapterId,
    pub volume_id: SignaturesVolumeId,
    pub records: List<SignaturesRecord, MAX_RECORDS_PER_CHAPTER>,
}

impl ChapterMethods<SignaturesSpec> for SignaturesChapter {
    fn volume_id(&self) -> &SignaturesVolumeId {
        &self.volume_id
    }

    fn chapter_id(&self) -> &SignaturesChapterId {
        &self.chapter_id
    }

    fn records(&self) -> &Vec<SignaturesRecord> {
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

    fn new_empty(volume_id: &SignaturesVolumeId, chapter_id: &SignaturesChapterId) -> Self {
        SignaturesChapter {
            chapter_id: chapter_id.clone(),
            volume_id: volume_id.clone(),
            records: List::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct SignaturesChapterId {
    pub val: Vector<u8, BYTES_FOR_SIGNATURE_CHARS>,
}

impl ChapterIdMethods<SignaturesSpec> for SignaturesChapterId {
    fn from_interface_id(id_string: &str) -> Result<Self> {
        let string = id_string.trim_start_matches("signatures_0x");
        let bytes = hex::decode(string)?;
        Ok(SignaturesChapterId {
            val: Vector::from_iter(bytes),
        })
    }

    fn interface_id(&self) -> String {
        format!("signatures_0x{}", self.as_string())
    }

    fn nth_id(n: u32) -> Result<SignaturesChapterId> {
        if n as usize >= SignaturesSpec::NUM_CHAPTERS {
            bail!("'n' must be <= NUM_CHAPTERS")
        }
        let byte_vec = vec![n as u8];
        Ok(SignaturesChapterId {
            val: Vector::from_iter(byte_vec),
        })
    }
}

impl SignaturesChapterId {
    /// Returns the ChapterId as a hex string (no 0x prefix).
    pub fn as_string(&self) -> String {
        hex::encode(&self.val)
    }
    /// Returns true if the candidate string starts with the ChapterId.
    pub fn matches(&self, candidate: &str) -> bool {
        candidate.starts_with(&self.as_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, SimpleSerialize)]
pub struct SignaturesVolumeId {
    /// Refers to the first signature in the Volume.
    ///
    /// It is global index of the address
    /// where all volumes are ordered oldest to youngest.
    ///
    /// ## Example
    ///
    /// The first signature in the first volume is 0, the first signature in the
    /// second volume is 1000 (derived from SIGNATURES_PER_VOLUME).
    pub first_signature: u32,
}

const VOL_PREFIX: &str = "mappings_starting_";

impl VolumeIdMethods<SignaturesSpec> for SignaturesVolumeId {
    fn from_interface_id(interface_id: &str) -> Result<Self> {
        let Ok(first_signature) = interface_id
            .trim_start_matches(VOL_PREFIX)
            .replace('_', "")
            .parse::<u32>()
            else {
                bail!("The string: {} was not formatted as expected.", interface_id)};

        Ok(SignaturesVolumeId { first_signature })
    }

    fn interface_id(&self) -> String {
        format!(
            "{}{}",
            VOL_PREFIX,
            utils::string::num_as_triplet(self.first_signature)
        )
    }

    fn nth_id(n: u32) -> Result<SignaturesVolumeId> {
        Ok(SignaturesVolumeId {
            first_signature: n * SIGNATURES_PER_VOLUME as u32,
        })
    }

    fn is_nth(&self) -> Result<u32> {
        Ok(self.first_signature / SIGNATURES_PER_VOLUME as u32)
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct SignaturesRecord {
    pub key: SignaturesRecordKey,
    pub value: SignaturesRecordValue,
}

impl RecordMethods<SignaturesSpec> for SignaturesRecord {
    fn key(&self) -> &SignaturesRecordKey {
        &self.key
    }

    fn value(&self) -> &SignaturesRecordValue {
        &self.value
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct SignaturesRecordKey {
    key: Vector<u8, BYTES_PER_SIGNATURE>,
}

impl RecordKeyMethods for SignaturesRecordKey {
    fn summary_string(&self) -> Result<String> {
        Ok(hex::encode(&self.key))
    }
}

impl SignaturesRecordKey {
    pub fn from_signature(address: &str) -> Result<Self> {
        if address.len() != 8 {
            bail!("Signature provided must be 8 characters long.")
        }
        let raw_bytes = hex::decode(address)?;
        Ok(Self {
            key: Vector::from_iter(raw_bytes),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct SignaturesRecordValue {
    pub texts: List<Text, MAX_TEXTS_PER_RECORD>,
}

impl Display for SignaturesRecordValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::from("Text(s):");
        let strings = match self.texts_as_strings() {
            Ok(s) => s,
            Err(_) => return Err(std::fmt::Error),
        };
        for t in strings {
            s.push(' ');
            s.push_str(&t)
        }
        write!(f, "{}", s)?;
        Ok(())
    }
}

impl RecordValueMethods for SignaturesRecordValue {
    fn summary_strings(&self) -> Result<Vec<String>> {
        let t = format!("texts: {:?}", self.texts_as_strings()?);
        Ok(vec![t])
    }
}

impl SignaturesRecordValue {
    /// Turns SSZ bytes into a vector of readable strings.
    pub fn texts_as_strings(&self) -> Result<Vec<String>> {
        let mut s = vec![];
        for n in &self.texts.to_vec() {
            s.push(n.to_utf8_string()?)
        }
        Ok(s)
    }
}

#[derive(Clone, Debug, Default, PartialEq, SimpleSerialize)]
pub struct Text {
    pub val: List<u8, MAX_BYTES_PER_TEXT>,
}

impl Text {
    pub fn from_string(s: &str) -> Self {
        Text {
            val: List::from_iter(s.as_bytes().to_vec()),
        }
    }
    pub fn to_utf8_string(&self) -> Result<String> {
        let v = self.val.to_vec();
        let s = from_utf8(&v)?;
        Ok(s.to_string())
    }
}

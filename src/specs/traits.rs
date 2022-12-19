use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::path::PathBuf;
use std::str::FromStr;
use tree_hash::TreeHash;

use crate::extraction::traits::Extractor;
use crate::samples::traits::SampleObtainer;

// Placeholder for the real trait.
pub trait SszDecode {}

/// An alias for common Traits most things should implement.
///
/// Saves manually writing them out for each associated type.
///
/// - Clone + Debug
/// - Default
/// - PartialEq + PartialOrd + Hash
/// - Serialize + Deserialize
///
/// Related but not included:
/// - Send + Sync + Unpin
/// - Eq + Ord
pub trait UsefulTraits<'a>:
    Clone + Debug + Default + PartialEq + PartialOrd + Hash + Serialize + Deserialize<'a>
{
}
impl<'a, T> UsefulTraits<'a> for T where
    T: Clone + Debug + Default + PartialEq + PartialOrd + Hash + Serialize + Deserialize<'a>
{
}

pub trait UsefulTraits2<'a>:
    Clone + Debug + Default + PartialEq + Serialize + Deserialize<'a>
{
}
impl<'a, T> UsefulTraits2<'a> for T where
    T: Clone + Debug + Default + PartialEq + Serialize + Deserialize<'a>
{
}

pub trait SszTraits: Encode + Decode + TreeHash {}
impl<T> SszTraits for T where T: Encode + Decode + TreeHash {}

/// A trait for specifying a new type of data.
///
/// Any data source that will be transformed by min-know must implement this trait.
///
/// # Example
///
/// ```ignore
/// impl DataSpec for AddressIndexSpec {
///     const NUM_CHAPTERS = 256;
///     // --snip--
/// }
/// ```
/// # Terms
/// - Chapter (holds record_key/record_value pairs). One chapter per Vol/Chapter pair.
/// - RecordKey: Something a user has/knows (address) and uses to get more information
/// - RecordValue: The data that is the result of a record_key (appearance transactions).
/// - raw_pair: a raw_key-raw_value pair where key can be turned into a record_key
/// and value becomes an record_value.
/// - raw_key (unformatted record_key)
/// - raw_value (unformatted record_value)
pub trait DataSpec: Sized {
    const NUM_CHAPTERS: usize;
    const MAX_VOLUMES: usize;
    // Associated types. They must meet certain trait bounds. (Alias: Bound).

    type AssociatedChapter: ChapterMethods<Self> + for<'a> UsefulTraits2<'a> + Send + Sync;
    type AssociatedChapterId: ChapterIdMethods<Self> + for<'a> UsefulTraits2<'a> + Send + Sync;
    type AssociatedVolumeId: VolumeIdMethods<Self> + for<'a> UsefulTraits<'a> + Send + Sync;

    type AssociatedRecord: RecordMethods<Self> + for<'a> UsefulTraits2<'a>;
    type AssociatedRecordKey: RecordKeyMethods + for<'a> UsefulTraits2<'a>;
    type AssociatedRecordValue: RecordValueMethods + for<'a> UsefulTraits2<'a>;

    type AssociatedExtractor: Extractor<Self>;
    type AssociatedSampleObtainer: SampleObtainer;

    type AssociatedManifest: ManifestMethods<Self> + for<'a> UsefulTraits2<'a>;
    /// Returns the enum variant that represents the spec for the database.
    ///
    /// This is used in coordinating platform-specific directories. It ensures
    /// that all implementations of the spec also create a new enum variant.
    fn spec_name() -> SpecId;
    /// Returns the version of the specification for the particular database.
    fn spec_version() -> String;
    /// Returns the number of Chapters that the spec defines.
    fn num_chapters() -> usize {
        Self::NUM_CHAPTERS
    }
    /// Returns the string representing the specification.
    ///
    /// For example, a CID or a URL.
    fn spec_schemas_resource() -> String;
    /// Gets all possible ChapterIds for a given spec.
    ///
    /// This is used when creating a new database, where chapters can be created
    /// in parallel.
    fn get_all_chapter_ids() -> Result<Vec<Self::AssociatedChapterId>> {
        (0..Self::NUM_CHAPTERS)
            .map(|n| Self::AssociatedChapterId::nth_id(n as u32))
            .collect()
    }
    /// Gets a vector of all the VolumeIds as defined by the available raw data.
    fn get_all_volume_ids(raw_data_path: &PathBuf) -> Result<Vec<Self::AssociatedVolumeId>> {
        let latest_vol = Self::AssociatedExtractor::latest_possible_volume(raw_data_path)?;
        let latest_vol_position = Self::AssociatedVolumeId::is_nth(&latest_vol)?;
        // Loop and get nth_id
        (0..=latest_vol_position)
            .map(|n| Self::AssociatedVolumeId::nth_id(n as u32))
            .collect()
    }
    fn record_key_to_volume_id(record_key: Self::AssociatedRecordKey) -> Self::AssociatedVolumeId;
    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> Result<Self::AssociatedChapterId>;
    /// Used to check the key for a piece of raw data when creating new database.
    fn record_key_matches_chapter(
        record_key: &Self::AssociatedRecordKey,
        vol: &Self::AssociatedVolumeId,
        chapter: &Self::AssociatedChapterId,
    ) -> bool;
    /// Coerces record_key into the type required for the spec.
    fn raw_key_as_record_key(key: &str) -> Result<Self::AssociatedRecordKey>;
    /// Some unformatted data that needs to be converted to an record_value
    /// to then be appended to a Chapter.record_values vector.
    fn raw_value_as_record_value<T>(raw_data_value: T) -> Self::AssociatedRecordValue;
    //fn new_chapter() -> Self::AssociatedChapter;
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub enum SpecId {
    #[default]
    AddressAppearanceIndex,
    Sourcify,
    FourByte,
}

/// Marker trait. Allows each db spec to define a struct
/// for what a volume ID is, and mark it with this trait.
///
/// ## Example
/// Specs can use any definition, as long as they mark
/// it with the trait.
///
/// A volume defined using an SSZ fixed vector.
/// ```
/// # use ssz_types::{FixedVector, typenum::U2};
/// # use min_know::specs::types::VolumeIdMethods;
/// struct VolumeId1 {
///     val: FixedVector<u8, U2>
/// }
/// impl VolumeIdMethods for VolumeId1 {}
/// ```
/// A volume defined using an integer.
/// ```
/// # use min_know::specs::types::VolumeIdMethods;
/// struct VolumeId2 {
///     detail: u32
/// }
/// impl VolumeIdMethods for VolumeId2 {}
/// ```
/// ## Rationale
/// The generic functions in database/types.rs use a set of
/// marker traits to define common functions.
pub trait VolumeIdMethods<T: DataSpec>: Sized {
    /// Returns the VolumeId for the given interface id.
    fn from_interface_id(interface_id: &str) -> Result<Self>;
    /// Returns the interface id for the Volume.
    fn interface_id(&self) -> String;
    /// Returns the VolumeId for the zero-based n-th Volume.
    ///
    /// Volumes are arranged lexicographically from 0 to n-1, where
    /// n is the latest volume..
    ///
    /// # Example
    /// If there are 100 volumes, then:
    /// - n=0 returns the first VolumeId
    /// - n=99 returns the last VolumeId
    fn nth_id(n: u32) -> Result<T::AssociatedVolumeId>;
    /// The zero-based position for the given VolumeId.
    ///
    /// If volume ids are placed in lexicographical order, corresponds to
    /// the position in that sequence. First position is n=0.
    fn is_nth(&self) -> Result<u32>;
    /// Gets all the VolumeIds earlier than and including the given VolumeId.
    ///
    /// E.g. If the Volume has a zero-based index of 10, returns 11 VolumeIds (0, 1, 2, ... 10).
    fn all_prior(&self) -> Result<Vec<T::AssociatedVolumeId>> {
        let mut vols: Vec<T::AssociatedVolumeId> = vec![];
        let last = self.is_nth()?;
        for n in 0..=last {
            let vol = Self::nth_id(n)?;
            vols.push(vol)
        }
        Ok(vols)
    }
}
pub trait ChapterIdMethods<T: DataSpec>: Sized {
    /// Returns the ChapterId from an interface id.
    fn from_interface_id(id_string: &str) -> Result<Self>;
    /// Returns the interface id for the Chapter.
    fn interface_id(&self) -> String;
    /// Returns the ChapterId for the zero-based n-th Chapter.
    ///
    /// Chapters are arranged lexicographically from 0 to n-1, where
    /// n is the total `NUM_CHAPTERS`.
    ///
    /// # Example
    /// If `NUM_CHAPTERS` is 256, then:
    /// - n=0 returns the first
    /// - n=255 returns the last ChapterId
    ///
    /// # Error
    /// Returns an error if n is outside range: `[0, NUM_CHAPTERS - 1]`.
    fn nth_id(n: u32) -> Result<T::AssociatedChapterId>;
    /// Derives a ChapterId from a chapter directory
    ///
    /// A chapter directory contains only files with the same chapter id.
    /// It is named using the interface id.
    fn from_chapter_directory(dir_path: &PathBuf) -> Result<Self> {
        let Some(chap_dir_name) = dir_path.file_name() else {
            bail!("Couldn't read dir name {:?}.", dir_path)};
        let Some(chapter_name) = chap_dir_name.to_str() else {
            bail!("Couldn't parse dir name {:?}.", chap_dir_name)};
        let id = Self::from_interface_id(chapter_name)?;
        Ok(id)
    }
}

/// Marker trait.
pub trait RecordKeyMethods {
    /// Returns the key struct that implements this method.
    fn get(self) -> Self;
}
pub trait RecordValueMethods {
    /// Returns the value struct that implements this method.
    fn get(self) -> Self;
    fn as_strings(self) -> Vec<String>;
}

/// Marker trait.
pub trait RecordMethods<T: DataSpec> {
    /// Returns the key struct that implements this method.
    fn get(&self) -> &Self;
    fn new(key: T::AssociatedRecordKey, val: T::AssociatedRecordValue) -> T::AssociatedRecord;
    /// Get the RecordKey of the Record.
    fn key(&self) -> &T::AssociatedRecordKey;
    /// Get the RecordValues of the Record.
    fn values_as_strings(self) -> Vec<String>;
}
/// Methods for the smallest distributable chapter in the database.
///
/// This refers to the pieces that can be looked up in the manifest
/// and shared over a network. It can be thought of as a "volume
/// chapter". The structure is different for each kind of database.
///
/// # Example
/// Address appearance index: Addresses (and their transactions)
/// for a specific volume and chapter.
///
/// Sourficy: Contract metadata for a specific volume and chapter.
pub trait ChapterMethods<T: DataSpec> {
    /// Returns the key struct that implements this method.
    fn get(self) -> Self;
    // An input that a user can provide to retrieve useful information.
    //
    // Each database is designed around the premise that a user has
    // some information that they want to look up in the database.
    //
    // # Examples
    // For an address apeparance database, the record_key is an address.
    //
    // For an ABI database, the record_key is a contract identifier.
    fn find_record(&self, key: T::AssociatedRecordKey) -> T::AssociatedRecord;
    /// Get the VolumeId.
    ///
    /// The method likely just returns the relevant struct member, which is
    /// otherwise inacessible to methods generic over T.
    fn volume_id(&self) -> &T::AssociatedVolumeId;
    /// Get the ChapterId.
    ///
    /// The method likely just returns the relevant struct member, which is
    /// otherwise inacessible to methods generic over T.
    fn chapter_id(&self) -> &T::AssociatedChapterId;
    /// Gets all the records present in the Chapter.
    fn records(&self) -> &Vec<T::AssociatedRecord>;
    /// Chapter struct as byte representation for storage.
    ///
    /// This allows databases to have custom methods (SSZ, SSZ+snappy, etc.)
    fn as_serialized_bytes(&self) -> Vec<u8>;
    /// Chapter struct from byte representation from storage.
    ///
    /// This allows databases to have custom methods (SSZ, SSZ+snappy, etc.)
    fn from_file(data: Vec<u8>) -> Result<Self>
    where
        Self: Sized;
    /// The filename of the chapter
    fn filename(&self) -> String;
    fn new_empty(volume_id: &T::AssociatedVolumeId, chapter_id: &T::AssociatedChapterId) -> Self;
}

/// Methods for the manifest of the database.
///
/// This refers to the object that contains the metadata that
/// will form the JSON-encoded manifest.
///
/// The manifest is required to contain some specific data, including
/// as the IPFS CID for each chapter. Other data may be added as needed
/// for any given database.
pub trait ManifestMethods<T: DataSpec> {
    /// Returns the version string.
    fn spec_version(&self) -> &str;
    /// Sets the version string.
    fn set_spec_version(&mut self, version: String);
    /// Returns the schemas string that can be used to acquire the spec
    /// for the database.
    fn schemas(&self) -> &str;
    /// Sets the schemas string that can be used to acquire the spec
    /// for the database.
    fn set_schemas(&mut self, schemas: String);
    /// Returns the id of the database.
    fn database_interface_id(&self) -> &str;
    /// Adds the database interface id.
    ///
    /// This value originates from the DataKind enum. Its value
    /// may depend on configuration choices, such as network.
    fn set_database_interface_id(&mut self, id: String);
    /// Returns the id of the most recent volume.
    fn latest_volume_identifier(&self) -> &str;
    /// Sets the interface identifier of the latest volume.
    fn set_latest_volume_identifier(&mut self, volume_interface_id: String);
    /// Returns the CIDs for all Chapters.
    fn cids(&self) -> Result<Vec<(&str, T::AssociatedVolumeId, T::AssociatedChapterId)>>;
    /// Sets the CIDs for all Chapters to the Manifest.
    ///
    /// CID: v0 IPFS Content Identifiers (CID). CIDs are all paired with
    /// Volume and Chapter Ids so that their interface ids can be stored
    /// alongside each CID. E.g., (CID, volume_interface_id, chapter_interface_id)
    /// can be grouped in the manifest.
    fn set_cids<U: AsRef<str> + Display>(
        &mut self,
        cids: &[(U, T::AssociatedVolumeId, T::AssociatedChapterId)],
    );
}

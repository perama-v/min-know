use anyhow::Result;
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;
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

    fn spec_name() -> SpecId;
    fn num_chapters() -> usize {
        Self::NUM_CHAPTERS
    }
    /// Gets all possible ChapterIds for a given spec.
    ///
    /// This is used when creating a new database, where chapters can be created
    /// in parallel.
    fn get_all_chapter_ids() -> Result<Vec<Self::AssociatedChapterId>> {
        (0..Self::NUM_CHAPTERS)
            .map(|n| Self::AssociatedChapterId::nth_id(n as u32))
            .collect()
    }
    /// Gets a vector of all the VolumeIds
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
    fn new_chapter() -> Self::AssociatedChapter;
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
pub trait VolumeIdMethods<T: DataSpec> {
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
}
pub trait ChapterIdMethods<T: DataSpec> {
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
    /// Chapter struct from byte representation for storage.
    ///
    /// This allows databases to have custom methods (SSZ, SSZ+snappy, etc.)
    fn from_file(data: Vec<u8>) -> Result<Self>
    where
        Self: Sized;
    /// The filename of the chapter
    fn filename(&self) -> String {
        format!(
            "{}_{}.ssz",
            self.volume_id().interface_id(),
            self.chapter_id().interface_id()
        )
    }
}

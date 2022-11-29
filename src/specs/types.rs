use anyhow::Result;
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use std::fmt::Debug;
use std::hash::Hash;
use tree_hash::TreeHash;

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
pub trait DataSpec {
    const DATABASE_INTERFACE_ID: &'static str;
    const NUM_CHAPTERS: usize;
    const MAX_VOLUMES: usize;
    // Associated types. They must meet certain trait bounds. (Alias: Bound).
    type AssociatedVolumeId: VolumeIdMethods + for<'a> UsefulTraits<'a>;
    type AssociatedChapterId: ChapterIdMethods + for<'a> UsefulTraits2<'a>;

    type AssociatedChapter: ChapterMethods + for<'a> UsefulTraits2<'a>;

    type AssociatedRecord: RecordMethods + for<'a> UsefulTraits2<'a>;
    type AssociatedRecordKey: RecordKeyMethods + for<'a> UsefulTraits2<'a>;
    type AssociatedRecordValue: RecordValueMethods + for<'a> UsefulTraits2<'a>;

    fn spec_name() -> SpecId;
    fn num_chapters() -> usize {
        Self::NUM_CHAPTERS
    }
    fn volume_interface_id<T>(volume: T) -> String;
    fn chapter_interface_id<T>(chapter: T) -> String;
    fn get_all_chapter_ids() -> Vec<Self::AssociatedChapterId>;
    fn get_all_volume_ids() -> Vec<Self::AssociatedVolumeId>;
    fn record_key_to_volume_id(record_key: Self::AssociatedRecordKey) -> Self::AssociatedVolumeId;
    fn record_key_to_chapter_id(
        record_key: Self::AssociatedRecordKey,
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
    //fn new_record(key: Self::AssociatedRecordKey, val: Self::AssociatedRecordValue) -> Self::AssociatedRecord;
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
pub trait VolumeIdMethods {}
pub trait ChapterIdMethods {
    // TODO
    /// Returns the id for the Chapter.
    fn interface_id(&self) -> String;
    /// Returns the directory name for the Chapter.
    fn dir_name(&self) -> String;
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
pub trait RecordMethods {
    /// Returns the key struct that implements this method.
    fn get(self) -> Self;
    fn new<T: DataSpec>(key: T::AssociatedRecordKey, val: T::AssociatedRecordValue) -> T::AssociatedRecord;
    fn key<T: DataSpec>(&self) -> T::AssociatedRecordKey;
    /// Values
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
pub trait ChapterMethods {
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
    fn find_record<T: DataSpec>(&self, key: T::AssociatedRecordKey) -> T::AssociatedRecord;
    fn volume_id<T: DataSpec>(&self) -> T::AssociatedVolumeId;
    fn chapter_id<T: DataSpec>(&self) -> T::AssociatedChapterId;
    fn records<T: DataSpec>(&self) -> Vec<T::AssociatedRecord>;
    /// Chapter struct as byte representation for storage.
    ///
    /// This allows databases to have custom methods (SSZ, SSZ+snappy, etc.)
    fn as_serialized_bytes(&self) -> Vec<u8>;
    /// Chapter struct from byte representation for storage.
    ///
    /// This allows databases to have custom methods (SSZ, SSZ+snappy, etc.)
    fn from_file<T: DataSpec>(data: Vec<u8>) -> Result<T::AssociatedChapter>;
}

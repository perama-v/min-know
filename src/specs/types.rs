use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

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

pub trait UsefulTraitsSszFriendly<'a>:
    Clone + Debug + Default + PartialEq + Serialize + Deserialize<'a>
{
}
impl<'a, T> UsefulTraitsSszFriendly<'a> for T where
    T: Clone + Debug + Default + PartialEq + Serialize + Deserialize<'a>
{
}

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
    type AssociatedChapterId: ChapterIdMethods + for<'a> UsefulTraits<'a>;
    type AssociatedChapter: ChapterMethods + for<'a> UsefulTraits<'a>;

    type AssociatedRecordKey: RecordKeyMethods;
    type AssociatedRecordValue: RecordValueMethods + for<'a> UsefulTraitsSszFriendly<'a>;

    fn spec_name() -> SpecId;
    fn num_chapters() -> usize {
        Self::NUM_CHAPTERS
    }
    fn volume_interface_id<T>(volume: T) -> String;
    fn chapter_interface_id<T>(chapter: T) -> String;
    fn get_all_chapter_ids() -> Vec<Self::AssociatedChapterId>;
    fn get_all_volume_ids() -> Vec<Self::AssociatedVolumeId>;
    fn record_key_to_volume_id(record_key: Self::AssociatedRecordKey) -> Self::AssociatedVolumeId;
    fn record_key_to_chapter_id(record_key: Self::AssociatedRecordKey) -> Self::AssociatedChapterId;
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
}

pub enum SpecId {
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
/// struct VolumeId1 {
///     val: FixedVector<u8, 2>
/// }
/// impl VolumeIdMarker for VolumeId1 {}
/// ```
/// A volume defined using an integer.
/// ```
/// struct VolumeId2 {
///     detail: u32
/// }
/// impl VolumeIdMarker for VolumeId2 {}
/// ```
/// ## Rationale
/// The generic functions in database/types.rs use a set of
/// marker traits to define common functions.
pub trait VolumeIdMethods {}
pub trait ChapterIdMethods {
    // TODO
    // Returns the id for a given Chapter
    // fn to_string() -> String;
}
/// Marker trait.
pub trait RecordKeyMethods {}
pub trait RecordValueMethods {}

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
    // An input that a user can provide to retrieve useful information.
    //
    // Each database is designed around the premise that a user has
    // some information that they want to look up in the database.
    //
    // # Examples
    // For an address apeparance database, the record_key is an address.
    //
    // For an ABI database, the record_key is a contract identifier.
    fn record_key<T>(_value: T) {}
    // Get the volume identifier for the chapter.
    //
    // E.g., Some block number or an counter.
    //fn volume_interface_id() -> String;
    // Get the chapter identifier for the chapter if applicable.
    //
    // E.g., Some "0xf6", or "contract_0xacbd..." or None.
    //fn chapter_interface_id() -> Option<String>;
}
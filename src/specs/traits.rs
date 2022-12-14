use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::config::choices::DataKind;
use crate::extraction::traits::ExtractorMethods;
use crate::samples::traits::SampleObtainerMethods;

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
pub trait UsefulTraits:
    Clone + Debug + Default + PartialEq + PartialOrd + Hash + Send + Sync
{
}
impl<T> UsefulTraits for T where
    T: Clone + Debug + Default + PartialEq + PartialOrd + Hash + Send + Sync
{
}

pub trait BasicUsefulTraits: Clone + Debug + Default + PartialEq + Send + Sync {}
impl<T> BasicUsefulTraits for T where T: Clone + Debug + Default + PartialEq + Send + Sync {}

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

    // Associated types. They must meet certain trait bounds. (Alias: Bound).

    type AssociatedChapter: ChapterMethods<Self> + BasicUsefulTraits;
    type AssociatedChapterId: ChapterIdMethods<Self> + BasicUsefulTraits;
    type AssociatedVolumeId: VolumeIdMethods<Self> + UsefulTraits;

    type AssociatedRecord: RecordMethods<Self> + BasicUsefulTraits;
    type AssociatedRecordKey: RecordKeyMethods + BasicUsefulTraits;
    type AssociatedRecordValue: RecordValueMethods + BasicUsefulTraits;

    type AssociatedExtractor: ExtractorMethods<Self>;
    type AssociatedSampleObtainer: SampleObtainerMethods;

    type AssociatedManifest: ManifestMethods<Self>
        + BasicUsefulTraits
        + Serialize
        + for<'de> Deserialize<'de>;
    /// Checks if the enum variant matches the spec for the database.
    ///
    /// This is used in coordinating platform-specific directories. It ensures
    /// that all implementations of the spec also create a new enum variant.
    fn spec_matches_input(data_kind: &DataKind) -> bool;
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
    fn get_all_volume_ids(raw_data_path: &Path) -> Result<Vec<Self::AssociatedVolumeId>> {
        let latest_vol = Self::AssociatedExtractor::latest_possible_volume(raw_data_path)?;
        let latest_vol_position = Self::AssociatedVolumeId::is_nth(&latest_vol)?;
        // Loop and get nth_id
        (0..=latest_vol_position)
            .map(|n| Self::AssociatedVolumeId::nth_id(n))
            .collect()
    }
    /// Gets the ChapterId relevant for a key.
    ///
    /// ## Example
    /// An address 0xabcd...1234 might return the ChapterId matching "ab"
    ///
    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> Result<Self::AssociatedChapterId>;
    /// Coerces a key string into the RecordKey type required for the spec.
    ///
    /// ## Example
    /// If the key is a hex string, it might convert that to
    /// a struct capable of ssz encoding.
    fn raw_key_as_record_key(key: &str) -> Result<Self::AssociatedRecordKey>;
}

/**
Allows each db spec to define a struct
for what a volume ID is, and mark it with this trait.

## Example
Specs can use any definition, as long as they mark
it with the trait.

A volume defined using an SSZ fixed vector.
```ignore
# use ssz_types::{FixedVector, typenum::U2};
# use min_know::specs::traits::VolumeIdMethods;
struct VolumeIdOne {
    val: FixedVector<u8, U2>
}
impl VolumeIdMethods<DataSpecOne> for VolumeIdOne {
    todo!()
}
```
A volume defined using an integer.
```ignore
# use ssz_types::{FixedVector, typenum::U2};
# use min_know::specs::traits::VolumeIdMethods;
struct VolumeIdTwo {
    detail: u32
}
impl VolumeIdMethods<DataSpecTwo> for VolumeIdTwo {
    todo!()
}
```
## Rationale
The generic functions in database/types.rs use a set of
marker traits to define common functions.
*/
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
    /// ```sh
    /// n=0, id=0
    /// n=1, id=100_000
    /// n=2, id=200_000
    /// let oldest_block = n * BLOCKS_PER_VOLUME;
    /// ```
    fn nth_id(n: u32) -> Result<T::AssociatedVolumeId>;
    /// The zero-based position for the given VolumeId.
    ///
    /// If volume ids are placed in lexicographical order, corresponds to
    /// the position in that sequence. First position is n=0.
    ///
    /// ## Example
    /// Here the calculation is simply to divide by the number of blocks
    /// per Volume:
    /// ```sh
    /// id=0, n=0
    /// id=100_000, n=1
    /// id=200_000, n=2
    /// -> self.oldest_block / BLOCKS_PER_VOLUME
    /// ```
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

/// Methods that RecordKeys must implement.
pub trait RecordKeyMethods {
    /// Returns the key as a String.
    ///
    /// ## Example
    /// For example, a basic hex to string conversion:
    /// ```sh
    /// hex::encode(self.key.to_vec())
    /// ```
    fn summary_string(&self) -> Result<String>;
}

/// Methods that RecordValues must implement.
pub trait RecordValueMethods {
    /// Returns the value, with all elements as Strings in a vector.
    fn summary_strings(&self) -> Result<Vec<String>>;
}

/// Marker trait.
pub trait RecordMethods<T: DataSpec> {
    /// Get the RecordKey of the Record.
    fn key(&self) -> &T::AssociatedRecordKey;
    /// Get the RecordValue of the Record.
    fn value(&self) -> &T::AssociatedRecordValue;
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
    fn as_serialized_bytes(&self) -> Result<Vec<u8>>;
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
    fn cids(&self) -> Result<Vec<ManifestCids<T>>>;
    /// Sets the CIDs for all Chapters to the Manifest.
    ///
    /// ## Example
    /// Pass a tuples of the form: (CID, volume_interface_id, chapter_interface_id).
    /// These will be grouped together in the manifest.
    ///
    /// CID: v0 IPFS Content Identifiers (CID).
    ///
    /// CIDs are all paired with
    /// Volume and Chapter Ids so that their interface ids can be stored
    /// alongside each CID.
    fn set_cids<C>(&mut self, cids: &[(C, T::AssociatedVolumeId, T::AssociatedChapterId)])
    where
        C: AsRef<str> + Display;
}

pub struct ManifestCids<T: DataSpec> {
    pub(crate) cid: String,
    pub(crate) volume_id: T::AssociatedVolumeId,
    pub(crate) chapter_id: T::AssociatedChapterId,
}

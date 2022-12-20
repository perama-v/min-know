use std::path::Path;

use anyhow::Result;

use crate::specs::traits::DataSpec;

/// A new database must implement this trait.
///
/// It provides the content of the database in a standard
/// interface.
///
/// Each database has different algorithms for turning raw data into
/// TODD-compliant data. Each database must provide a
/// type that implements this trait.
pub trait Extractor<T: DataSpec> {
    /// Returns a formed Chapter using raw data in the provided source directory.
    ///
    /// Returns `None` if the are no source files that match the provided IDs.
    /// This may be the case when processing sample data.
    fn chapter_from_raw(
        chapter_id: &T::AssociatedChapterId,
        volume_id: &T::AssociatedVolumeId,
        source_dir: &Path,
    ) -> Result<Option<T::AssociatedChapter>>;
    /// Returns the VolumeId of the latest possible volume that can be made from
    /// the available raw data.
    ///
    /// ## Example
    /// If volumes are produce every 100 units of data (0-99, 100-199, ...),
    /// and the raw data has 340 units. Then the latest will exclude the
    /// incomplete 40 and return the id for volume 200-299.
    fn latest_possible_volume(source_dir: &Path) -> Result<T::AssociatedVolumeId>;
}

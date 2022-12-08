use std::path::PathBuf;

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
    /// E.g.,
    fn chapter_from_raw(
        chapter_id: &T::AssociatedChapterId,
        volume_id: &T::AssociatedVolumeId,
        source_dir: &PathBuf,
    ) -> Result<T::AssociatedChapter>;
}

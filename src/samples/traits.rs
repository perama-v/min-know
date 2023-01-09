use std::path::Path;

use anyhow::Result;
/// Gets samples for the given todd database.
///
/// Processed samples are TODD-style samples. Raw samples
/// are data that can be transformed/processed into a TODD-style
/// format.
///
/// Each database has different samples, and must provide a
/// type that implements this trait.
pub trait SampleObtainerMethods {
    /// Returns the filenames that are raw samples.
    ///
    /// Used to check if the samples are present. These filenames
    /// are known in advance and must be hard coded.
    fn raw_sample_filenames() -> Vec<&'static str>;
    /// Returns the volume interface ids for volumes that
    /// are represented in the samples.
    ///
    /// Return `None` and the samples will be created from raw samples.
    /// The volumes created can then be inspected and included here.
    fn sample_volumes() -> Option<Vec<&'static str>>;
    /// Detects if processed samples are present at the given location.
    fn get_raw_samples(dir: &Path) -> Result<()>;
}

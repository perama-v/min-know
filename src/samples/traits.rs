use std::path::{Path, PathBuf};

use anyhow::Result;
/// Gets samples for the given todd database.
///
/// Processed samples are TODD-style samples. Raw samples
/// are data that can be transformed/processed into a TODD-style
/// format.
///
/// Each database has different samples, and must provide a
/// type that implements this trait.
pub trait SampleObtainer {
    /// Returns the filenames that are raw samples.
    ///
    /// Used to check if the samples are present.
    fn raw_sample_filenames() -> Vec<&'static str>;
    /// Returns the filenames that are processed samples.
    ///
    /// Used to check if the samples are present.
    ///
    /// Return `None` and the samples will be created from raw samples.
    /// Some of those files can then be selected and included here.
    fn processed_sample_filenames() -> Option<Vec<&'static str>>;
    /// Detects if processed samples are present at the given location.
    fn get_raw_samples(dir: &PathBuf) -> Result<()>;
}

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
    /// Detects if raw samples are present at the given location.
    fn raw_samples_present(dir: &PathBuf) -> bool;
    /// Detects if processed samples are present at the given location.
    fn processed_samples_present(dir: &PathBuf) -> bool;
    /// Gets raw samples.
    fn get_raw_samples(dir: &PathBuf) -> Result<()>;
}

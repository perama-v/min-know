/// Gets samples for the given todd database.
pub trait SampleObtainer {
    /// Gets samples that can be transformed
    /// into the todd database format.
    fn get_raw_samples() {}
    /// Gets samples in the database format.
    fn get_todd_samples() {}
}
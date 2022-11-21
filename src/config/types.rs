// E.g., path::unchainedpath
pub trait SourceDataPath {}

// E.g., path::addressindexpath
pub trait DestinationDataPath {}

pub trait DataName {
    fn name() -> String;
}

/// The starting point for setting up a new database.
///
/// # Example
/// ```
/// impl Config for AddressAppearanceIndex
/// ```
pub trait DatabaseConfig {
    type Name: DataName;
    type Source: SourceDataPath;
    type Destination: DestinationDataPath;
}

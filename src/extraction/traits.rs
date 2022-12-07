/// A new database must implement this trait.
///
/// It provides the content of the database in a standard
/// interface.
///
/// Each database has different algorithms for turning raw data into
/// TODD-compliant data. Each database must provide a
/// type that implements this trait.
pub trait Extractor {
    /// Returns an iterator for data that matches a chapter.
    /// The data may be then iterated over to match against
    /// different volumes.
    ///
    /// # Example
    /// If the source database is the Unchained Index, returns
    /// an iterator of addresses starting with the same two characters.
    fn get_all_for_chapter(volume: u32, chapter: u32) {}
}

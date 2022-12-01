/// A new source database must implement this trait.
///
/// It provides the content of the database in a standard
/// interface.
pub trait SourceParser {
    /// Returns an iterator for data that matches a chapter.
    /// The data may be then iterated over to match against
    /// different volumes.
    ///
    /// # Example
    /// If the source database is the Unchained Index, returns
    /// an iterator of addresses starting with the same two characters.
    fn chapter(volume: u32, chapter: u32) {}
}

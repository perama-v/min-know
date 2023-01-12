/// Number of additions present in a single Volume.
///
/// This is a count of addresses, and includes additions to
/// addresses already present in the database.
pub const ENTRIES_PER_VOLUME: usize = 1_000;

/// Derived from ENTRIES_PER_VOLUME.
pub const MAX_RECORDS_PER_CHAPTER: usize = ENTRIES_PER_VOLUME;

/// Number of bytes required to describe a chapter. As defined in the spec.
///
/// https://github.com/perama-v/TODD/blob/main/example_specs/nametag.md
pub const BYTES_FOR_ADDRESS_CHARS: usize = 1;

/// Number of bytes for an address.
pub const BYTES_PER_ADDRESS: usize = 20;

pub const MAX_BYTES_PER_NAME: usize = 32;

pub const MAX_BYTES_PER_TAG: usize = 32;

pub const MAX_TAGS_PER_RECORD: usize = 256;

pub const MAX_NAMES_PER_RECORD: usize = 256;

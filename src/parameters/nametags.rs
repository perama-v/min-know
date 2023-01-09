use ssz_types::typenum::{U1, U20, U256, U32};

/// Number of additions present in a single Volume.
///
/// This is a count of addresses, and includes additions to
/// addresses already present in the database.
pub const ENTRIES_PER_VOLUME: u32 = 1000;

/// Number of bytes required to describe a chapter. As defined in the spec.
///
/// https://github.com/perama-v/TODD/blob/main/example_specs/nametag.md
pub type BytesForAddressChars = U1;

/// Number of bytes for an address.
pub type BytesPerAddress = U20;

pub type MaxBytesPerName = U32;

pub type MaxBytesPerTag = U32;

pub type MaxTagsPerRecord = U256;

pub type MaxNamesPerRecord = U256;

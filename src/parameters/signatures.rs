//! Paramaters are defined in the spec
//!
//! See: https://github.com/perama-v/TODD/blob/main/example_specs/nametag.md
/// Number of additions present in a single Volume.
///
/// This is a count of hex signatures, and includes additions to
/// signatures already present in the database.
pub const SIGNATURES_PER_VOLUME: usize = 1_000;

/// Derived from ENTRIES_PER_VOLUME.
pub const MAX_RECORDS_PER_CHAPTER: usize = SIGNATURES_PER_VOLUME;

/// Number of bytes required to describe a chapter.
pub const BYTES_FOR_SIGNATURE_CHARS: usize = 1;

/// Number of bytes for signature.
pub const BYTES_PER_SIGNATURE: usize = 4;

pub const MAX_BYTES_PER_TEXT: usize = 256;

pub const MAX_TEXTS_PER_RECORD: usize = 256;

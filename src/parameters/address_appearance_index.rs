//! Constants used in the library.

/// Number of blocks in a group of appearances. Data is stored in discrete ranges
/// so that as the chain progresses it is clear how new appearances are to be stored.
///
/// E.g., for RANGE = 100_000
///     - [0, 99_999]
///     - [100_000, 199_999]
///     - ...
pub const BLOCKS_PER_VOLUME: u32 = 100_000;

/// Number of hex characters that address within a [chapter][0] share.
///
/// Alias for [ADDRESS_CHARS_SIMILARITY_DEPTH][1]. A depth of `2` indicates
/// that addresses "0x3eab" and "0x3e56" are similar and belong to chapter "3e".
///
/// [0]: https://github.com/perama-v/address-appearance-index-specs#design-parameters
/// [1]: (https://github.com/perama-v/address-appearance-index-specs#design-parameters)
pub const ADDRESS_CHARS_SIMILARITY_DEPTH: u32 = 2;

/// Number of bytes per address. Value may be different in some networks.
///
/// For EVM-based chains this is usually 20 bytes. Used by types::NetworkConfig.
pub const DEFAULT_BYTES_PER_ADDRESS: usize = 20;

/// Number of valid ASCII bytes a network name may use.
pub const MAX_NETWORK_NAME_BYTES: u32 = 32;

const HEX_BASE: u32 = 16;

/// Number of chapters.
pub const NUM_CHAPTERS: u32 = HEX_BASE.pow(ADDRESS_CHARS_SIMILARITY_DEPTH);

/// This type is defined in the [specification][1].
///
/// # Typed Number
/// `Un` is the number `n`, not an `n`-bit integer. It is a helper type
/// for ssz operations.
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#constants
pub const MAX_ADDRESSES_PER_VOLUME: usize = 1073741824;

/// Derived from MAX_ADDRESSES_PER_VOLUME
///
/// If the all the data was in one chapter, this is the maximum size of that Chapter.
pub const MAX_RECORDS_PER_CHAPTER: usize = MAX_ADDRESSES_PER_VOLUME;

/// This type is defined in the [specification][1].
///
/// # Typed Number
/// `Un` is the number `n`, not an `n`-bit integer. It is a helper type
/// for ssz operations.
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#constants
pub const MAX_TXS_PER_VOLUME: usize = 1073741824;

/// This type is defined in the [specification][1].
///
/// # Typed Number
/// `Un` is the number `n`, not an `n`-bit integer. It is a helper type
/// for ssz operations.
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#constants
pub const NUM_COMMON_BYTES: usize = 1;

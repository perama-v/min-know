//! Constants used in the library.

/// Number of blocks in a group of appearances. Data is stored in discrete ranges
/// so that as the chain progresses it is clear how new appearances are to be stored.
///
/// E.g., for RANGE = 100_000
///     - [0, 99_999]
///     - [100_000, 199_999]
///     - ...
pub const BLOCK_RANGE_WIDTH: u32 = 100_000;

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
pub const DEFAULT_BYTES_PER_ADDRESS: u32 = 20;

/// Number of valid ASCII bytes a network name may use.
pub const MAX_NETWORK_NAME_BYTES: u32 = 32;

/// Specification major [version][1]
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#indexmanifest
pub const SPEC_VER_MAJOR: u32 = 0;

/// Specification minor [version][1]
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#indexmanifest
pub const SPEC_VER_MINOR: u32 = 1;

/// Specification patch [version][1]
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#indexmanifest
pub const SPEC_VER_PATCH: u32 = 0;

const HEX_BASE: u32 = 16;

/// Number of chapters.
pub const NUM_CHAPTERS: u32 = HEX_BASE.pow(ADDRESS_CHARS_SIMILARITY_DEPTH);

/// String representing a location or address that can be used to obtain the
/// address-appearance-index [specification][1].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#indexspecificationschemas
pub const SPEC_RESOURCE_LOCATION: &str = "https://github.com/perama-v/address-appearance-index-specs";

/// String prefix of the address-appearance-index manifest [publishing topic][1].
///
/// The network name must be concatenated (e.g., "address_appearance_index_mainnet").
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#indexpublishingidentifier
pub const PUBLISHING_PREFIX: &str = "address_appearance_index_";
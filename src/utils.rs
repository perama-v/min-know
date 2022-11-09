//! Utility functions including string manipulation.
use anyhow::anyhow;
use regex::Regex;

use crate::{
    constants::{ADDRESS_CHARS_SIMILARITY_DEPTH, BLOCK_RANGE_WIDTH, SPEC_VER_MAJOR},
    unchained::types::BlockRange,
};

/// Name of the directory that contains an [address chapter][1].
///
/// E.g., Input `chapter` of "4e" (without "0x").
/// Contains .ssz_snappy files that represent [address index volumes][2].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#addresschapter
/// [2]: https://github.com/perama-v/address-appearance-index-specs#addressindexvolume
pub fn chapter_dir_name(chapter: &str) -> String {
    let chapter_name = format!("chapter_0x{}", chapter);
    chapter_name
}

/// Extracts a chapter identifier from a chapter directory name.
///
/// E.g., a name: "chapter_0x4e" returns "4e" as byte vector.
pub fn chapter_dir_to_id(chapter_dir: &str) -> Result<Vec<u8>, anyhow::Error> {
    let hex = chapter_dir.trim_start_matches("chapter_0x");
    let bytes = hex::decode(hex)?;
    Ok(bytes)
}

/// Name of the file that contains an [address index volume][1].
///
/// E.g., Input `first_block` 12_100_000.
/// Contains .ssz_snappy files that represent [address index volumes][2].
///
/// [1]: https://github.com/perama-v/address-appearance-index-specs#addresschapter
/// [2]: https://github.com/perama-v/address-appearance-index-specs#addressindexvolume
pub fn volume_file_name(chapter: &str, first_block: u32) -> Result<String, anyhow::Error> {
    let volume = num_to_name(first_block);
    let chapter_name = format!("chapter_0x{}_volume_{:0<9}.ssz_snappy", chapter, volume);
    Ok(chapter_name)
}

/// Converts a number into a readable format for filenames.
///
/// E.g., block number 14500000 becomes "014_500_000".
pub fn num_to_name(block_number: u32) -> String {
    let mut name = format!("{:0>9}", block_number);
    for i in [6, 3] {
        name.insert(i, '_');
    }
    name
}

/// Converts a custom filename into a block number.
///
/// E.g., file "chapter_0x4e_volume_014_500_000" becomes 14500000.
pub fn name_to_num(filename: &str) -> Result<u32, anyhow::Error> {
    let parts = Regex::new(
        r"(?x)
    (?P<first>\d{3})  # the first three.
    _
    (?P<mid>\d{3}) # the middle three.
    _
    (?P<last>\d{3}) # the last three.
    ",
    )?
    .captures(filename)
    .ok_or_else(|| {
        anyhow!(
            "file {} title lacks a number of the form xxx_xxx_xxx",
            filename
        )
    })?;
    let num_string = format!("{}{}{}", &parts["first"], &parts["mid"], &parts["last"]);
    let num: u32 = num_string.parse()?;
    Ok(num)
}

/// Returns the first hex characters of an address to a DEPTH.
pub fn address_to_chapter(address: &str) -> Result<String, anyhow::Error> {
    let address = address.to_lowercase();
    let chapter = address
        .trim_start_matches("0x")
        .get(0..ADDRESS_CHARS_SIMILARITY_DEPTH as usize)
        .ok_or_else(|| anyhow!("Address doesn't have enough characters."))?;
    if !chapter.is_ascii() {
        return Err(anyhow!(
            "Address {} starts with unexpected characters.",
            address
        ));
    }
    Ok(chapter.to_string())
}

/// For the oldest block in a volume, returns the `BlockRange` that volume covers.
///
/// The range is inclusive of the two bounds.
///
/// # Example
/// If `RANGE=100_000`:
/// ```
/// # use min_know::utils::volume_id_to_block_range;
///
/// let range = volume_id_to_block_range(15_400_000)?;
/// assert_eq!(range.old, 15_400_000);
/// assert_eq!(range.new, 15_499_999);
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn volume_id_to_block_range(oldest_block: u32) -> Result<BlockRange, anyhow::Error> {
    let newest_block = oldest_block + BLOCK_RANGE_WIDTH - 1;
    let range = BlockRange::new(oldest_block, newest_block)?;
    Ok(range)
}

/// Converts a hex string to byte vector
///
/// Left pads with a zero if there are an odd number of characters.
pub fn hex_string_to_bytes(input: &str) -> Result<Vec<u8>, anyhow::Error> {
    let mut hex_input = String::from("");
    let trimmed = input.trim_start_matches("0x");
    if trimmed.len() % 2 != 0 {
        hex_input.push_str("0");
    }
    hex_input.push_str(trimmed);
    let bytes = hex::decode(hex_input)?;
    Ok(bytes)
}

#[test]
fn odd_hex_length() -> Result<(), anyhow::Error> {
    let input = "0x30e";
    let bytes = hex_string_to_bytes(input)?;
    let output = hex::encode(bytes);
    assert_eq!(
        input.trim_start_matches("0x").trim_start_matches("0"),
        output.as_str().trim_start_matches("0")
    );
    Ok(())
}

/// Determines if the spec version in the manifest filename is compatible with
/// the version in the library.
///
/// ## Errors
/// Returns an error in significant library<->data version discrepancies.
///
/// ### Significant (Error)
///
/// `Error`. Decoding and parsing data may result in errors.
///
/// - lib MAJOR > data MAJOR
///     - Lib has new breaking changes, such as SSZ, parameter or filename definition changes.
/// - data MAJOR > lib MAJOR
///     - Lib version is older than the one used to encode these data.
///
/// ### Not significant
///
/// `Ok()`. Changes are not likely to result in errors.
///
/// - lib MINOR > data MINOR
///     - Lib has new compatible changes, such as add-on features and helper functions.
/// - data MINOR > lib MINOR
///     - Lib version lacks new add-on features and helper functions in the spec. While
/// the data was encoded with these features, it is still compatible.
/// - data PATCH != lib PATCH
///
pub fn manifest_version_ok(filename: &str) -> Result<(), anyhow::Error> {
    let parts = Regex::new(
        r"(?x)
    (?P<major>\d{2})  # the first two.
    _
    (?P<minor>\d{2}) # the middle two.
    _
    (?P<patch>\d{2}) # the last two.
    ",
    )?
    .captures(filename)
    .ok_or_else(|| {
        anyhow!(
            "file {} title lacks a number of the form xx_xx_xx",
            filename
        )
    })?;
    let num_string = (&parts["major"]).to_string();
    let manifest_major: u32 = num_string.parse()?;
    if manifest_major != SPEC_VER_MAJOR {
        return Err(anyhow!(
            "Data spec (v{:02}.xx.xx) is incompatible with lib spec (v{:02}.xx.xx)",
            manifest_major,
            SPEC_VER_MAJOR
        ));
    }
    Ok(())
}

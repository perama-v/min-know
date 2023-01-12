/// This module holds the manifest-related structs.
///
// Dev note: The manifest may have different encoding requirements to the
// database data. For example, using serde::Serialize rather
// then SimpleSerialize to perform JSON encoding. Module
// separation makes procedural macros simple in this instance.
pub mod address_appearance_index;
pub mod nametags;

//! This module contains somple-obtaining methods that are unique to each database.
//!
//! The generic database manager can obtain samples, but requires that
//! each specification implement the necessary non-generic methods.
pub mod address_appearance_index;
pub mod nametags;
pub mod traits;

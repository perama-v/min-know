//! Each type of destination database that will be created needs a custom specification
//! for its structure.
//!
//! This is the the starting point for implementing a new database.
//!
//! A new spec struct can be made in a dedicated file here. Implement the specs/trait.rs/DataSpec
//! trait for that struct. The compiler will ask for implementations of traits and declaration
//! of associated types (such as a type for obtaining sample data).
//! This ensures that the generic machinery works for all additional databases added.
//!
//! ## Example
//! Create specs/mydatabase.rs and make a MySpec struct.
//! ```ignore
//! pub struct MyDatabaseSpec;
//! impl DataSpec for MyDatabaseSpec {}
//! ```
//! RustAnalyzer will offer to add the missing components of DataSpec. Do that, and then
//! keep going until the compiler is happy.
//!
//! Then you should be able to use the examples, replacing the example data spec struct with
//! your MyDatabaseSpec
pub mod address_appearance_index;
pub mod traits;

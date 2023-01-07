# Getting started

This is a guide for using min-know to turn an existing database into a TODD-compliant
database.

This is not a guide for using the library (see examples for this) - it is a guide for extending the library.

In the example below, this database will be called "MyData".

- [Getting started](#getting-started)
  - [End goal](#end-goal)
  - [Overview](#overview)
  - [Existing specs and data](#existing-specs-and-data)
  - [Begin](#begin)
  - [Provide types for the spec implementation](#provide-types-for-the-spec-implementation)
  - [Provide implementations for types](#provide-implementations-for-types)
  - [Replace `Associated*` descriptive types with actual types](#replace-associated-descriptive-types-with-actual-types)
  - [Replace `todo!()`'s](#replace-todos)
  - [Add parameters](#add-parameters)
  - [Add extractor](#add-extractor)
  - [Add sample handler](#add-sample-handler)
  - [Add MyData to choices](#add-mydata-to-choices)

## End goal

A user obtains a manifest fo "MyData" that someone created and published.
They then use the manifest to obtain relevant parts of the database and
download them from IPFS. They can then be queried and pinned back IPFS.
```rs
fn main() -> Result<()> {

    let db: Todd<MyDataSpec> = Todd::init(DataKind::MyData, DirNature::Sample)?;

    // A user has two things they would like to look up.
    let queries = ["user_query_abcde", "user_query_fghij"];

    // Obtain Chapters relevant to those queries using the manifest.
    static IPFS_GATEWAY_URL: &str = "https://127.0.0.1:8080";
    db.obtain_relevant_data(&addresses, IPFS_GATEWAY_URL)?;

    // Check out the data, comparing it to the manifest.
    let check = db.check_completeness()?;
    println!("Check result: {:?}", check);

    // Look something up in the database.
    let key = "user_query_abcde";
    let values = db.find(key)?;
    for v in values {
        println!("{:?}", v);
    }
    Ok(())
}

```

## Overview

Data can be turned into a TODD-compliant database by following the [specification](https://github.com/perama-v/TODD). This mainly deciding on:

- What does a user start with? (define a `RecordKey`)
- What does a user get? (define a `RecordValue`)
- How can a `Volume` be divided (define a `Chapter` definition)
- How often should `Volumes` be pulbished (define a `Volume` cadence)

Once those decisions are made, a new module can be added to this library.

```sh
./specs
    - address_appearance_index.rs (an existing spec for address appearances)
    - my_data_spec.rs
```

The heart of the library is the `DataSpec`. The type system will
auto-complete the components of the spec that you are required to complete.

## Existing specs and data

The [TODD spec](https://github.com/perama-v/TODD) is a meta-spec.
It allows other specs to be defined that are compliant with it.

Here are two examples, showing the relationship between raw data, a spec
that is TODD-compliant and the min-know code that transforms the raw data
into TODD-compliant data.

- Address appearance index data
    - raw data: https://trueblocks.io/papers/2022/file-format-spec-v0.40.0-beta.pdf
    - todd-compliant spec: https://github.com/perama-v/address-appearance-index-specs
    - todd-compliant code: [./src/specs/address_appearance_index.rs](./src/specs/address_appearance_index.rs)
- Contract names and tags data
    - raw data: https://github.com/perama-v/RolodETH/tree/main/data
    - todd-compliant spec: https://github.com/perama-v/TODD/blob/main/example_specs/nametag.md
    - todd-compliant code: unimplemented
- Contract metadata
    - raw data: https://sourcify.dev/
    - todd-compliant spec: unimplemented
    - todd-compliant code: unimplemented
- Contract log and function signatures
    - raw data: https://github.com/ethereum-lists/4bytes and https://www.4byte.directory/
    - todd-compliant spec: unimplemented
    - todd-compliant code: unimplemented

The common theme with the above data is that there are users who only want
part of the whole. Yet the whole also keeps growing as new data is added.

## Begin

To begin, write the following:

```rs
// In ./specs/my_data_spec.rs
use super::traits::DataSpec;

pub struct MyDataSpec {}

impl DataSpec for MyDataSpec {}
```
Add it to the specs module:
```rs
// In ./specs/mod.rs
pub mod my_data_spec
```
Then allow [Rust Analyzer](https://github.com/rust-lang/rust-analyzer)
in the IDE to auto-complete the required methods. For example in [VS Code](https://code.visualstudio.com/) the "[Quick Fix](https://code.visualstudio.com/docs/editor/refactoring)"
shortcut gives you the option to "Implement Default Members".

```rs
// In ./specs/my_data_spec.rs
pub struct MyDataSpec {}

impl DataSpec for MyDataSpec {
    const NUM_CHAPTERS: usize;

    type AssociatedChapter;
    /* snip */
}
```
## Provide types for the spec implementation

The new spec now has all the associated constants, associated types and methods it needs.

For constants, provide a value. For associated types provide a new empty struct.

```rs
// In ./specs/my_data_spec.rs
pub struct MyDataSpec {}

pub struct MyDataChapter {}

impl DataSpec for MyDataSpec {
    const NUM_CHAPTERS: usize = 4096;

    type AssociatedChapter = MyDataChapter;
    /* snip */
}
```

The easiest way to do this is to replace "`Associated`" with "`MyData`", where MyData
will be unique to your database.

## Provide implementations for types

Then continue to be guided as to what each struct requires. Here we are told
that `MyDataChapter` needs `ChapterMethods`.
```sh
the trait `specs::traits::ChapterMethods<MyDataSpec>` is not implemented for `MyDataChapter`
```
Provide the implementation by using QuickFix e.g., select "Generate trait impl for MyDataChapter"
```rs
// In ./specs/my_data_spec.rs
use super::traits::{DataSpec, ChapterMethods};

pub struct MyDataSpec {}

pub struct MyDataChapter {}

impl ChapterMethods<MyDataSpec> for MyDataChapter {}
/* snip */
```
Some `XyzMethods` impls require the "`<MyDataSpec>`", and rust analyzer will communicate this with
the following phrase:  missing generics for trait `specs::traits::XyzMethods`"

```rust
// Before: "Missing generics for trait specs::traits::ChapterMethods"
impl ChapterMethods for MyDataChapter {}

//After
impl ChapterMethods<MyDataSpec> for MyDataChapter {}
```

Then allow the methods to be auto completed using Quick Fix. Here we can see that
chapters require methods for getting a chapter id. Hovering over the function
gives you then documentation for what the function is supposed to do. The
other specs can also be inspected to see how they have implemented it.

```rs
// In ./specs/my_data_spec.rs
impl ChapterMethods<MyDataSpec> for MyDataChapter {
    fn chapter_id(&self) -> &<MyDataSpec as DataSpec>::AssociatedChapterId {
        todo!()
    }
    /* snip */

```

## Replace `Associated*` descriptive types with actual types

In `./specs/my_data_spec.rs`, anything that starts with `Associated*` can
be replaced with an actual type. After creating the types for MyDataSpec
and then implementing their required methods (auto-fill), replace
the descriptive types with the actual types. Both are technically correct,
but the latter are much easier to read.

The easiest way to do this is to generate all the types and implementations first,
then en-masse find-and-replace the following strings:

- "`<MyDataSpec as DataSpec>::Associated`" -> `MyData`
- "`T::Associated`" -> `MyData`
- (less commonly) "`T`" -> "`MyDataSpec`"

For example, the value being returned here is a little hard to read.

```rs
fn chapter_id(&self) -> &<MyDataSpec as DataSpec>::AssociatedChapterId
```
It is saying:

"It returns a reference to a ChapterId"

But the way it is saying it is as follows:

"It returns a reference to the ChapterId that is associated with your DataSpec"

```rs
// In ./specs/my_data_spec.rs
impl ChapterMethods<MyDataSpec> for MyDataChapter {
    fn chapter_id(&self) -> &<MyDataSpec as DataSpec>::AssociatedChapterId {
        todo!()
    }
    /* snip */

```

In implementing MyDataSpec we provide an AssociatedChapterId (just like for
AssociatedChapter). Once we create the some struct MyDataChapterId, and provide
the implementations it asks for (ChapterIdMethods), we can use that in this
functions return value:
```rs
// Before:
fn chapter_id(&self) -> &<MyDataSpec as DataSpec>::AssociatedChapterId
// After:
fn chapter_id(&self) -> &MyDataChapterId
```

Seeing these three types (spec, chapter and chapter id) and how they related to each other:
```rs
// In ./specs/my_data_spec.rs
pub struct MyDataSpec {}

pub struct MyDataChapter {}

pub struct MyDataChapterId {}

impl DataSpec for MyDataSpec {
    type AssociatedChapter = MyDataChapter;
    type AssociatedChapterId = MyDataChapterId;
    /* snip */
}

impl ChapterIdMethods for MyChapterId {
    /* snip */
}

impl ChapterMethods<MyDataSpec> for MyDataChapter {
    fn chapter_id(&self) -> &MyDataChapterId {
        todo!()
    }
    /* snip */
}
```

## Replace `todo!()`'s

Anywhere there is a todo, there needs to be code. These are things
that could not be performed generically. For example, one database
might have keys that are very different from keys in another
database.

## Add parameters

If parameters/constants are required for the database, they can be added
to the `./parameters` module:

```rs
// In ./parameters/mod.rs
pub mod my_data_spec
```


```rs
// In ./parameters/my_data_spec.rs
pub const MAX_THING: u32 = 42;
```

## Add extractor

The `AssociatedExtractor` extractor lives in a separate module because
it may require more code to implement.

```rs
// In ./extraction/mod.rs
pub mod my_data_spec
```

Provide a type and then implement the required methods, just like
in `./specs/my_data_spec.rs`
```rs
// In ./extraction/my_data_spec.rs
pub struct MyDataExtractor {}

impl Extractor<MyDataSpec> for MyDataExtractor {
    /* snip */
}
```
## Add sample handler


The `AssociatedSampleObtainer` sample handler lives in a separate module because
it may require more code to implement.

```rs
// In ./samples/mod.rs
pub mod my_data_spec
```

Provide a type and then implement the required methods, just like
in `./specs/my_data_spec.rs`
```rs
// In ./samples/my_data_spec.rs
pub struct MyDataSampleObtainer {}

impl SampleObtainer<MyDataSpec> for MyDataSampleObtainer {
    /* snip */
}
```

## Add MyData to choices

In `./config/choices.rs` there are some enums (`DataKind` and `DirNature`)
that are the basis for configuring the library for use.

Add MyData as an enum variant in `DataKind` then be guided to handle the
places where this new variant arises. For example, adding path configuration
in the `DirNature` implementation.
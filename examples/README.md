# Examples

The examples (`./example/*.rs`) here are for interacting with the min-know library.

They fall under the following categories:

- [Examples](#examples)
  - [`appearances_*.rs`](#appearances_rs)
  - [`nametags_*.rs`](#nametags_rs)
  - [`*_get_sample_data.rs`](#_get_sample_datars)
  - [`*_user_*.rs`](#_user_rs)
    - [Appearances: Where did an address appear?](#appearances-where-did-an-address-appear)
    - [Nametags: Whare are labels for an address?](#nametags-whare-are-labels-for-an-address)
  - [`*_maintainter_*.rs`](#_maintainter_rs)
    - [Appearances: Create the TODD-compliant database](#appearances-create-the-todd-compliant-database)
    - [Nametags: Create the TODD-compliant database](#nametags-create-the-todd-compliant-database)
  - [External](#external)

## `appearances_*.rs`

Examples for the address-appearance-index database. This contains
information about which transactions a given address appeared in.

## `nametags_*.rs`

Examples for the nametags database. This contains labels for addresses
in the form or names and tags ("nametags").

## `*_get_sample_data.rs`

Get sample data to be able to run the examples.

The example processes many transactions and are best run with `--release` tag.
```sh
cargo run --release --example appearances_get_sample_data
cargo run --release --example nametags_get_sample_data
```

## `*_user_*.rs`

Use the sample data from the perspective of an end user of a TODD-compliant database.


End user of the address-appearance-index. Someone with an address who wants to
find out which transactions their address appears in.
### Appearances: Where did an address appear?
```sh
cargo run --example appearances_user_1_obtain_part_of_db
cargo run --example appearances_user_2_check_local_db
cargo run --example appearances_user_3_query_db
```

### Nametags: Whare are labels for an address?
```sh
cargo run --example nametags_user_1_obtain_part_of_db
cargo run --example nametags_user_2_check_local_db
cargo run --example nametags_user_3_query_db
```

## `*_maintainter_*.rs`

Use the library to create an maintain a TODD-compliant database.

The examples process many transactions and are best run with `--release` tag.

### Appearances: Create the TODD-compliant database
```sh
cargo run --release --example appearances_maintainer_create_db
cargo run --release --example appearances_maintainer_extend_db
cargo run --release --example appearances_maintainer_generate_manifest
cargo run --release --example appearances_maintainer_repair_index
```

### Nametags: Create the TODD-compliant database
```sh
cargo run --release --example nametags_maintainer_create_db
cargo run --release --example nametags_maintainer_extend_db
cargo run --release --example nametags_maintainer_generate_manifest
cargo run --release --example nametags_maintainer_repair_index
```

## External

[PSR_B0943+10](https://github.com/perama-v/PSR_B0943_10) is a small application that
uses the sample data from the Address Appearance Index. It is
from the perspective of a Portal Node user who is trying
to get information about their wallet activity without using APIs.

This involves a TODD-compliant address appearance index and identifies
where APIs could be replaced with other TODD-compliant databases.


Notes on the layout and design of the crate.

```sh
# ./src
.
├── config # Directory handling. Some enums account for all databases here.
│   ├── address_appearance_index.rs
│   ├── choices.rs
│   ├── dirs.rs
│   └── mod.rs
├── database
│   ├── mod.rs
│   └── types.rs # Core generic machinery that does the
├── extraction # Broken out from specs.
│   ├── address_appearance_index.rs
│   ├── mod.rs
│   └── traits.rs
├── lib.rs
├── parameters
│   ├── address_appearance_index.rs
│   └── mod.rs
├── samples # Broken out from specs.
│   ├── address_appearance_index.rs
│   ├── mod.rs
│   └── traits.rs
├── specs
│   ├── address_appearance_index.rs # Custom database methods
│   ├── mod.rs
│   ├── my_data_spec.rs # Template for new databases to add methods
│   ├── nametags.rs # Custom database methods
│   └── traits.rs # ** MetaSpec from which specs are built **
└── utils
    ├── download.rs
    ├── ipfs.rs
    ├── mod.rs
    ├── system.rs
    └── unchained
```
## Overview
Min-know operates generically over databases. The main operations are in `./database`
and these are methods that apply to all databases. New databases don't need to touch this
directory.

Where databases require custom methods,
they are described as methods in `./specs/traits.rs`. So, in order to provide them,
a new database creates a new spec and implements those methods. For example:
`./specs/my_data_spec.rs` is a template where the instructions in
[GETTING_STARTED](./GETTING_STARTED.md) are followed.

## Naming
The way this is organised is by having types for logical parts of a database. Volumes, Chapters,
Keys, Values etc. These all follow naming schemes that are overly explicit. This is to assist
in the creation of new specs.

See how in the metaspec `./specs/traits.rs` the following appears:

```
type AssociatedRecordValue: RecordValueMethods + for<'a> BasicUsefulTraits<'a>;
```

The associated type is a RecordValue, and so it's called AssociatedRecordValue for clarity.

Similarly each associated type is defined by its methods. Hence, and AssociatedRecordValue
is defined by having RecordValueMethods (along with some other useful traits that can be
automatically derived with procedural macros.)

## Writing methods

Fundamentally, anything that cannot be generalised over all databases requires a method to be
written.

For example, the AssociateVolumeId type requires a an `interface_id()` method as follows:
```rust
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Hash, PartialOrd)]
pub struct MyDataVolumeId {
    // Different for every database.
    some_value: u32
}
/// Returns the interface id for the Volume.
fn interface_id(&self) -> String;
```

So because the VolumeId struct for different databases may have different values,
and each database defines its own interfacce identifier (as per [https://github.com/perama-v/TODD#interface-identifiers](https://github.com/perama-v/TODD#interface-identifiers)).
The conversion from the value to a String must be custom.

The generic machinery can then use that method as needed to get a string volume interface id,
irrespective of the database in question.
# Min-know

An implementation of the
[ERC-time-ordered-distributable-database](https://github.com/perama-v/TODD) (TODD)
as a generic library. It can be used to make data TODD-compliant to facilitate
peer-to-peer distribution.

> Status: prototype

- [Min-know](#min-know)
  - [Why does this library exist?](#why-does-this-library-exist)
  - [Principles](#principles)
  - [End Users](#end-users)
  - [Interface](#interface)
  - [Architecture](#architecture)
  - [Examples](#examples)
  - [Databases](#databases)
  - [Database Maintainers](#database-maintainers)
  - [Extend the library for your data](#extend-the-library-for-your-data)
  - [Manifest coordination using a smart contract](#manifest-coordination-using-a-smart-contract)
  - [Pin by default to IPFS](#pin-by-default-to-ipfs)
  - [Frequently Asked Questions](#frequently-asked-questions)
  - [Contributing](#contributing)

## Why does this library exist?

To test out a new database design, where user participation makes the entire database
more available.

Questions for you:
- Do you have data that grows over time and that you would like users
to host?
- Are you providing data as a public good and are wondering how to wean to community?

Min-know makes data into an append-only structure that anyone can publish to.
Distribution happens like a print publication where users obtain Volumes as they
are released. A user becomes a distributer too.

Volumes contain Chapters that can be obtained separately.
This effectively divides the database, making large databases manageable for
resource-constrained users.

## Principles

&#x1F4D8;&#x1F50D;&#x1F41F;

To make any database TODD-compliant so that data-users become data-providers.

TODD-compliance is about:
1. Delivering a user the *min*imum *know*ledge that is useful to them.
2. Delivering a user some extra data.
3. Making it easy for a user to become a data provider for the next user.

A minnow is a small fish &#x1F41F; that can be part of a larger collective.

## End Users

Data is published in `Volumes`.

&#x1F4D8; - A Volume

Volumes are added over time:

&#x1F4D8; &#x1F4D8; &#x1F4D8; &#x1F4D8; &#x1F4D8; ... &#x1F4D8; <--- &#x1F4D8; - All Volumes (published so far).

`Volumes` have `Chapters` for specific content. `Chapters` can be obtained individually.
- &#x1F4D8; An example volume with 256 `Chapters`
    - &#x1F4D5; `0x00` First chapter (1st)
    - ...
    - ...
    - &#x1F4D9; `0xff` Last Chapter (256th)

A `Manifest` &#x1F4DC; exists that lists all Chapters for all Volumes. A user can check the manifest and find which `Chapter` is right for
them.

&#x1F4DC;&#x1F50D;&#x1F41F;

The user starts with something they know (a key), for example, an address.
For every key, only one Chapter will be important.
- User (&#x1F41F;) key is an address: `0xf154...f00d`.
- Data is divided into chapters using the first two characters of address (`Chapter` = `0xf1`)

Visually:
- &#x1F4D5; `0x00`
- ...
- ...
- &#x1F4D7; `0xf1` <--- &#x1F41F; `0xf154...f00d` (user only needs this `Chapter`)
- ...
- ...
- &#x1F4D9; `0xff`

For every published `Volume`, the user only downloads the right `Chapter` for their needs.
The Min-know library automates this by using the CIDs in the manifest to find files on IPFS.

This means obtaining one `Chapter` from every `Volume` that has ever been published.
Hence, the user &#x1F41F; only needs 1/256th of the entire database.

Once downloaded, the `Chapters` can be queried for useful information that
the database contains.

Optionally, they can also pin their `Chapters` to IPFS, which makes the data
available from more sources.

## Interface

Iteraction with the library occurs the `Todd` struct ([`database::types::Todd`]) through the methods:

- For users:
    - `obtain_relevant_data()`
    - `check_completeness()`
    - `find()`
- For maintainers:
    - `full_transformation()`
    - `extend()`
    - `repair_from_raw()`
    - `generate_manifest()`
    - `manifest()`

## Architecture

See [./ARCHITECTURE.md](https://github.com/perama-v/min-know/blob/main/ARCHITECTURE.md)
for how this library is structured.

## Examples

All examples can be seen with the following command:

```sh
cargo run --example
```

See [./examples/README.md](https://github.com/perama-v/min-know/blob/main/examples/README.md) for more information.

## Databases

See [./DATABASES.md](https://github.com/perama-v/min-know/blob/main/DATABASES.md) for
different databases that have been implmemented in this library.

## Database Maintainers

The maintainer methods in the examples are used to create and extend a
TODD-compliant database.

This requires having a local "raw" source, which will be different for every
data type. The library will use the methods in the `./extraction` module
to convert the data.

For example:
- The address-appearance-index is created and maintained by
having locally available Unchained Index chunk files (produced by
trueblocks-core [https://github.com/TrueBlocks/trueblocks-core)](https://github.com/TrueBlocks/trueblocks-core)).
They are parsed and reorganised to form the TODD-compliant format.
- The nametags database is created and maintained by having individual files (one per address)
that contain JSON-encoded names and tags.

Other raw formats might be flat files containing data of various kinds.

## Extend the library for your data

See [./GETTING_STARTED.md](https://github.com/perama-v/min-know/blob/main/GETTING_STARTED.md) for how to use min-know for
a new database.

## Manifest coordination using a smart contract

TODD-compilance is about coordination by default (e.g., having a Schelling point for
a distributed database)

The manifest contains the CIDs of all the Chapters for a given database. A new
manifest is created when a database is updated and new CIDs are added. Old CIDs
remain unchanged.

After creating the manifest, that person can post it under their own
[IPNS](https://docs.ipfs.tech/concepts/ipns/#how-ipns-works). Anyone
who knows this IPNS can watch for new manifests to be published there.

To broadcast that you are going to publish, you can perform a single transaction
to a broadcasting contract ([https://github.com/perama-v/GAMB](https://github.com/perama-v/GAMB))
to record your
IPNS name with the topic you wish to publish (the name of the database you are
publishing).

After this single transaction, you can update your IPNS to the latest manifest
hash for free.

Anyone else can also submit their IPNS name to the contract and publish new
volumes for the database. While not yet implmemented, the process of
checking that contract, fetching manifests, comparing the CIDs they contain
and coordinating to collaborate on publishing can all be automated.

## Pin by default to IPFS

While not implmemented in this library, it is intended that end-users of a TODD-compliant
diatabase could automatically pin any Chapters they download. This could be an
opt out process and could result in most users contributing significantly to the
long term availability of data.

## Frequently Asked Questions

See [./FAQ.md](https://github.com/perama-v/min-know/blob/main/FAQ.md)

## Contributing

This is a very experimental library that is mostly an exploration for
feasibility analysis.

The library is not currently being used to deliver data to real end users.
Though it is designed to be readily implemented f
(see [./GETTING_STARTED.md](https://github.com/perama-v/min-know/blob/main/GETTING_STARTED.md)) that can all share the
same core of the library.

Does the idea interest you? A
suited for?

- twitter: @eth_worm
- github @perama-v

Raise an issue or say hi &#x2764;

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
  - [Examples](#examples)
    - [Address appearances index](#address-appearances-index)
  - [Database Maintainers](#database-maintainers)
  - [Extend the library for your data](#extend-the-library-for-your-data)
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

A user &#x1F41F; can check the manifest &#x1F50D; and find which `Chapter` is right for
them.

First, look at the key (query string) and match it to a `ChapterId`.

- &#x1F4D5; `0x00...`
- &#x1F4D7; `0x01...`
- ...
- &#x1F4D8; `0xf1...` <--- &#x1F41F; `0xf154...f00d` (only need this `Chapter`)
- ...
- &#x1F4D9; `0xff...`

Then download any `Chapter` that has that that `ChapterId`. This
is automated (the library uses IPFS to get files using the CID in the manifest).

This means obtaining a `Chapter` from every `Volume` that has ever been published.

&#x1F4D8; &#x1F4D8; &#x1F4D8; &#x1F4D8; &#x1F4D8; &#x1F4D8; ... <--- &#x1F4D8;

Once downloaded, the `Chapters` can be queried for useful information that
the database contains.

Optionally, they can also pin their `Chapters` to IPFS, which makes the data
available from more sources.


## Examples

All examples can be seen with the following command:

```sh
cargo run --example
```

See the [examples readme](./examples/README) for more information.

### Address appearances index

This is an index of which transactions an address was involved in.
It is a derivative of the Unchained Index
([spec pdf](https://trueblocks.io/papers/2022/file-format-spec-v0.40.0-beta.pdf)).

It can be used by user-facing software in post-EIP-4444 settings,
where chain history is distributed among peers.

It solves the problem of "what data should I request?" by
providing the transaction ids involved with the user's address.
Those transactions start the chain of requests for meaningful data.

The examples below are the counterpart to an exploration into
a tiny local wallet explorer. That exploration can be found in
this [blog post series](https://perama-v.github.io/ethereum/protocol/poking).

```sh
cargo run --example wallet_1_transaction_receipt
cargo run --example wallet_2_inspect_transaction_logs
cargo run --example wallet_3_decode_via_apis
```
The examples work toward delivering useful information about personal wallet
history, without using APIs and without using more than 1GB. That solution
starts by using chain data and a TODD-compliant address appearance database.

It also points to next steps for making event signatures, contract source code and
contract names and tags TODD-compliant.

## Database Maintainers

The maintainer methods in the examples are used to create and extend a
TODD-compliant database.

This requires having a local "raw" source, which will be different for every
data type. The library will use the methods in the `./extraction` module
to convert the data.

For example, the address-appearance-index is created and maintained by
having locally available Unchained Index chunk files (produced by
[trueblocks-core](https://github.com/TrueBlocks/trueblocks-core)).
They are parsed and reorganised to form the TODD-compliant format.

Other raw formats might be flat files containing data of various kinds.

## Extend the library for your data

See the [getting started guide](./GETTING_STARTED.md) for how to use min-know for
a new database.

## Contributing

This is a very experimental library that is mostly an exploration for
feasibility analysis.

The library is not currently being used to deliver data to real end users.
Though it is designed to be readily implemented for new data types
(see [getting started](./GETTING_STARTED.md)) that can all share the
same core of the library.

Does the idea interest you? Are there data types you think this might be
suited for? Raise an issue or get in touch :) @eth_worm

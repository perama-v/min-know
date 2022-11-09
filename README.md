# Min-know

An implementation of the
[address-appearance-index-spec](https://github.com/perama-v/address-appearance-index-specs).

> Status: prototype

## Purpose

&#x1F4D8;&#x1F50D;&#x1F41F;

What is the minimum knowledge that a small fish needs in order
to play?

Min-know can be used by user-facing software in post-EIP-4444 settings,
where chain history is distributed among peers.

It solves the problem of "what data should I request?" by
providing the transaction ids involved with the user's address.
Those transactions start the chain of requests for meaningful data.

## Mechanism

Min-know administers the address-appearance-index, which is a derivative
of the Unchained Index ([spec pdf](https://trueblocks.io/papers/2022/file-format-spec-v0.40.0-beta.pdf)).

Distribution and coordination may be achieved by using
[ERC-generalised-attributable-manifest-broadcaster](https://github.com/perama-v/GAMB)
compliant contracts.

The address-appearance-index has an
[ERC-time-ordered-distributable-database](https://github.com/perama-v/TODD)
compliant structure. Which means that disk requirements are small (<500MB) compared
to the whole index (~80GB). Content is designed to be shared in peer to peer networks.

As chain data grows, a new `Volume` containing recent transaction indices can be created
and published/broadcast.

&#x1F4DA; <- A `Volume` is published.

Volumes are composed of different `Chapters` that can be obtained separately.

## Index Users

A user &#x1F41F; can check the manifest &#x1F50D; and find which `Chapter` is right for
them by looking at what their address starts with:
- &#x1F4D5; `0x00...`
- &#x1F4D7; `0x01...`
- ...
- &#x1F4D8; `0xf1...` <--- &#x1F41F; `0xf154...f00d` (only need this `Chapter`)
- ...
- &#x1F4D9; `0xff...`

They add the latest `Chapter` to their existing collection of similar `Chapters` from old `Volumes`.

&#x1F4D8; &#x1F4D8; &#x1F4D8; &#x1F4D8; &#x1F4D8; &#x1F4D8; ... <--- &#x1F4D8;

Then they can start asking their post-history node (E.g., Portal node) for
those transactions.

Users can search for published manifests by looking them up at the contract deployed at:
```
0x0c316b7042b419d07d343f2f4f5bd54ff731183d
```
They need provide a publisher's
address, and the search topic: "address-appearance-index-mainnet"
(defined in the spec). A publisher could be anyone, so it may be most thorough
to scan the events in that contract looking for the desired topic and pulling
any publisher addresses from there.

Once a manifest CID has been found, they can fetch the manifest, find
the index `Chapters` that are important for their address, and download them.

Users may also pin any part that they have downloaded, and become
"miniature maintainers" themselves.

## Index Maintainers

The address-appearance-index may be created and extended by different people.

It is constructed by parsing the Unchained Index, which itself is constructed
by constructed by tracing transactions using
[trueblocks-core](https://github.com/TrueBlocks/trueblocks-core).
Maintainers can either:
- Run trueblocks' `chifra scrape`
against their own `trace_`-enabled archive node to create a the Unchained Index,
- Run trueblocks' `chifra init` to obtain it from peers over IPFS.

Once the index has been made, it may be published by broadcasting the manifest
IPFS CID at contract deployed at:
```
0x0c316b7042b419d07d343f2f4f5bd54ff731183d
```
This is an [ERC-generalised-attributable-manifest-broadcaster](https://github.com/perama-v/GAMB).
The topic "address-appearance-index-mainnet" is to be used for mainnet broadcasts.

## Examples

Check out the [example readme](./examples/README.md) and the files in `./examples`.

In general, the approach is to set up an IndexConfig based on the
location of your data:

```rust
// let data_dir = AddressIndexPath::Default;
let path = AddressIndexPath::Sample;

// let unchained = UnchainedPath::Default;
let unchained = UnchainedPath::Sample;
let network = Network::default();

let index = IndexConfig::new(&path, &network);
```

There is a method that sets up a sample data in
standard platform-specific directories.
```rust
index.get_sample_data(&unchained_path).await?;
```

Then call the main operations using `index.<command>`
```rust
// User
let manifest = index.read_manifest()?;
let appearances = index.find_transactions(address)?;

// Maintainer
index.maintainer_create_index(&unchained_path)?;
index.maintainer_extend_index(&unchained_path)?;
```

The `UnchainedPath::Default` path reads from
existing Unchained Index data that has been created using trueblocks-core
default settings and `chifra init`. The derived index data will
be placed in a new, separate directory defined in
`AddressIndexPath::Default`.

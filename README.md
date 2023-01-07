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
  - [Manifest Coordination using a smart contract](#manifest-coordination-using-a-smart-contract)
  - [Pin by default to IPFS](#pin-by-default-to-ipfs)
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
([https://trueblocks.io/papers/2022/file-format-spec-v0.40.0-beta.pdf](https://trueblocks.io/papers/2022/file-format-spec-v0.40.0-beta.pdf)).
That is to say, it is a reorganisation of the Unchained Index into
a "Volumes and Chapters" publishing model.

It can be used by user-facing software in post-EIP-4444 settings,
where chain history is distributed among peers.

It solves the problem of "what data should I request?" by
providing the transaction ids involved with the user's address.
Those transactions start the chain of requests for meaningful data.

The examples below are the counterpart to an exploration into
a tiny local wallet explorer. That exploration can be found in
this blog post series: [https://perama-v.github.io/ethereum/protocol/poking](https://perama-v.github.io/ethereum/protocol/poking).

Use of the the index can be seen in a demo application here:
https://github.com/perama-v/PSR_B0943_10

### Nametags

This is a database consisting of names and tags (collectively "nametags") for addresses.
In the source/raw data, each address is a file containing JSON-encoded data. For example:

```json
# cat ./data/0xffff03817c70c99a3eba035c4f851b2be6faee44
{
  "tags": [
    "contract-deployer"
  ],
  "name": "HitBTC Token: Deployer"
}
```

The size of this database is 2.7GB (720,000 addresses) and is likely a subset of the total data
available from the community.
The purpose of TODD-ify-ing the database is to allow small parts (2700/256 = 10MB)
to be individually accessed. Additionally to enable new names and tags to be added to the
database by different parties.

Publishers/maintainers can add additional nametags for addresses. This takes an existing manifest
and a directory of raw nametag files. The extend method in min-know will check each file
and if the nametag is not already present, adds it to the next Volume to be published.

- What does a user start with? (define a `RecordKey`)
    - A address.
- What does a user get? (define a `RecordValue`)
    - Names and Tags
- How can a `Volume` be divided (define a `Chapter` definition)
    - By address starting characters (0x00 - 0xff), which equates to 256 cChapters per Volume.
- How often should `Volumes` be pulbished (define a `Volume` cadence)
    - Every 10,000 new address additions (E.g., there would be ~72 editions to date). 
    - This includes appending new nametags to addresses already in the database.

## Database Maintainers

The maintainer methods in the examples are used to create and extend a
TODD-compliant database.

This requires having a local "raw" source, which will be different for every
data type. The library will use the methods in the `./extraction` module
to convert the data.

For example, the address-appearance-index is created and maintained by
having locally available Unchained Index chunk files (produced by
trueblocks-core [https://github.com/TrueBlocks/trueblocks-core)](https://github.com/TrueBlocks/trueblocks-core)).
They are parsed and reorganised to form the TODD-compliant format.

Other raw formats might be flat files containing data of various kinds.

## Extend the library for your data

See the [getting started guide](./GETTING_STARTED.md) for how to use min-know for
a new database.

## Manifest Coordination using a smart contract

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

## Contributing

This is a very experimental library that is mostly an exploration for
feasibility analysis.

The library is not currently being used to deliver data to real end users.
Though it is designed to be readily implemented for new data types
(see [getting started](./GETTING_STARTED.md)) that can all share the
same core of the library.

Does the idea interest you? Are there data types you think this might be
suited for? Raise an issue or get in touch :) @eth_worm

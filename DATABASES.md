# Databases

The following are descriptions of different databases that have been
implemented in min-know to perform transformation into the TODD-compliant form.

- [Databases](#databases)
  - [Address appearances index](#address-appearances-index)
  - [Nametags](#nametags)
    - [General framework](#general-framework)
    - [Tradeoffs](#tradeoffs)


## Address appearances index

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

### Manifest
This is the first few lines of the manifest for the sample data
```json
{
  "spec_version": "0.1.0",
  "schemas": "https://github.com/perama-v/address-index/tree/main/address_appearance_index",
  "database_interface_id": "address_appearance_index_mainnet",
  "latest_volume_identifier": "volume_014_400_000",
  "chapter_cids": [
    {
      "volume_interface_id": "volume_011_200_000",
      "chapter_interface_id": "chapter_0x00",
      "cid_v0": "QmaGbMhGwC2tnCHmiu3xtnLbdWquNC7VT5U4kQGLxy6qEh"
    },
    {
      "volume_interface_id": "volume_011_200_000",
      "chapter_interface_id": "chapter_0x01",
      "cid_v0": "QmfRS4DBAHodAcJhUYJ8RDN2YqNLxcdwqgW5pxsh4rjjhA"
    },
    {
      "volume_interface_id": "volume_011_200_000",
      "chapter_interface_id": "chapter_0x02",
      "cid_v0": "QmfUjasa9ePSFJa9cryUZDy8ifeGhiBE1R67JPcPCAKWUw"
    },
    ...
    {
      "volume_interface_id": "volume_014_400_000",
      "chapter_interface_id": "chapter_0xff",
      "cid_v0": "QmWUZp2ZHoVhkhdFSkBnyynpv6PREGZgU49QFMcdxegsvz"
    }
  ]
}
```
## Nametags

### General framework

This is a database consisting of names and tags (collectively "nametags") for addresses.
In the source/raw data, each address is a file containing JSON-encoded data.

For example, the address `0xffff03817c70c99a3eba035c4f851b2be6faee44` has the following
entry:
```json
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
    - By address starting characters (0x00 - 0xff), which equates to 256 Chapters per Volume.
- How often should `Volumes` be pulbished (define a `Volume` cadence)
    - Every 1,000 new address additions (E.g., there would be ~720 editions to date).
    - This includes appending new nametags to addresses already in the database.

### Tradeoffs

Note that the source database has additional data that is not included in the new database:

```json
{
    "description",
    "erc20_symbol",
    "twitterUsername",
    "name",
    "tags",
    "imageURL",
    "erc721_symbol",
    "externalUrl",
    "proxy",
    "imageUrl",
    "discordUrl",
    "erc20_decimals",
    "ens"
}
```

Variation 1 parameters: 256 Chapters, Volumes with 1000 addresses (720_000/1_000 = 720).
|Status|Dirs|Files|Footprint|
|-|-|-|-|
|DB Pre|1|720_000|2.8 GB|
|DB Post|256|184_320 (256*720)|722 MB|
|Manifest|-|-|32 MB|
|End user|-|720|2.8 MB|

Comments:
- The new data occupies 1/4th the original size, factors include:
    - Only "names" and "tags" from the original data are included.
    - The data is SSZ-encoded, which may be more efficient than JSON.
- The manifest is large (32 MB) because the volumes are small and there are many chapters.

Variation 2 parameters: 16 Chapters, Volumes with 1000 addresses (720_000/1_000 = 720).
|Status|Dirs|Files|Footprint|
|-|-|-|-|
|DB Pre|1|720_000|2.8 GB|
|DB Post|16|11_520 (16*720)|? 722 MB|
|Manifest|-|-|? 2 MB|
|End user|-|720|? 44.8 MB|

Comments:
- Untested
- Fewer chapters make for larger user footprint, but easier to check manifest.


Variation 3 parameters: 256 Chapters, Volumes with 10_000 addresses (720_000/10_000 = 72).
|Status|Dirs|Files|Footprint|
|-|-|-|-|
|DB Pre|1|720_000|2.8 GB|
|DB Post|256|18_432 (256*72)|? 722 MB|
|Manifest|-|-|? 3.2 MB|
|End user|-|72|? 28 MB|

Comments:
- Untested
- Larger Volumes (10_000 addresses) makes publishing less frequent. If
someone has <10_000 they cannot publish, unless they find more, by waiting
or coordinating to get more data.

### Manifest
This is the first few lines of the manifest for the sample data
```json
{
  "spec_version": "0.1.0",
  "schemas": "https://github.com/perama-v/TODD/blob/main/example_specs/nametag.md",
  "database_interface_id": "nametags",
  "latest_volume_identifier": "nametags_from_000_001_000",
  "chapter_cids": [
    {
      "volume_interface_id": "nametags_from_000_000_000",
      "chapter_interface_id": "addresses_0x00",
      "cid_v0": "QmQtTcM2RnitEgrJCz9NUE4vEoxPh5XX1ZQyFSopun7iAL"
    },
    {
      "volume_interface_id": "nametags_from_000_000_000",
      "chapter_interface_id": "addresses_0x01",
      "cid_v0": "QmfHPw1gttig7n3V1os9UTpYagXxZCdaTAUM2HZLqhFG2z"
    },
    {
      "volume_interface_id": "nametags_from_000_000_000",
      "chapter_interface_id": "addresses_0x02",
      "cid_v0": "QmYmJvkz6SF5LRwCF234SH9222ohzUXnYETHZPV35a8i9Q"
    },
    {
      "volume_interface_id": "nametags_from_000_000_000",
      "chapter_interface_id": "addresses_0x03",
      "cid_v0": "QmPmw1AUst3Zm3oci8KBPmMZqCsmn4VKFe112zFWC16VAv"
    },
    {
      "volume_interface_id": "nametags_from_000_000_000",
      "chapter_interface_id": "addresses_0x04",
      "cid_v0": "QmQAezhkitssjybrpt6NydR6FeAPjhfFq1pvebyWLESTHq"
    },
    ...
    {
      "volume_interface_id": "nametags_from_000_001_000",
      "chapter_interface_id": "addresses_0xff",
      "cid_v0": "QmPbUjpauoCwCDeLiuzkBC1WPTFErj6ieKTiGGjF84jCi3"
    }
  ]
}
```
## Signatures

The source/raw database consists of 534_574 files (2.1 GB). Filenames are four byte hex signatures (allowed have 20 byte filenames), containing text that represent the source
of the signature (`keccak(contents)`)

The database allows a user to start with a signature and get useful text:
```sh
"dd62ed3e" => "allowance(address,address)"
 ^ signature   ^ text
```
Raw data samples (./samples/todd_signatures/raw_source_signatures)
consists of 2099 files sampled from
https://github.com/ethereum-lists/4bytes/tree/master/signatures.
It includes files that have collisions (text is delineated by ';' within those files.)

Publishers/maintainers can add additional text for signatures. This takes an existing manifest
and a directory of raw signature files. The extend method in min-know will check each file
and if the text is not already present, adds it to the next Volume to be published.

- What does a user start with? (define a `RecordKey`)
    - A signature (4 byte hex string, e.g., `dd62ed3e`).
- What does a user get? (define a `RecordValue`)
    - Text (source of the signature e.g., `allowance(address,address)`)
- How can a `Volume` be divided (define a `Chapter` definition)
    - By signature starting characters (0x00 - 0xff), which equates to 256 Chapters per Volume.
- How often should `Volumes` be pulbished (define a `Volume` cadence)
    - Every 1,000 new signature additions (E.g., there would be ~530 editions to date from this
    database alone).
    - This includes appending new text for signatures already in the database.

### Manifest
This is the first few lines of the manifest for the sample data
```json
{
  "spec_version": "0.1.0",
  "schemas": "https://github.com/perama-v/TODD/blob/main/example_specs/signatures.md",
  "database_interface_id": "signatures",
  "latest_volume_identifier": "mappings_starting_000_001_000",
  "chapter_cids": [
    {
      "volume_interface_id": "mappings_starting_000_000_000",
      "chapter_interface_id": "signatures_0x00",
      "cid_v0": "QmdrYLVjdqh58AbNaAWifGn8YUinUWJyFRgdQkKecSzdyx"
    },
    {
      "volume_interface_id": "mappings_starting_000_000_000",
      "chapter_interface_id": "signatures_0x01",
      "cid_v0": "QmZ9nrpnk2DsDpWTSmXdZXYdrWow8rAGUQDJiY78CZk56h"
    },
    {
      "volume_interface_id": "mappings_starting_000_000_000",
      "chapter_interface_id": "signatures_0x02",
      "cid_v0": "QmSnpf5J6JyziyPn9ncZfp7gYwFraMquSdW32wvGYKNw4v"
    },
    {
      "volume_interface_id": "mappings_starting_000_000_000",
      "chapter_interface_id": "signatures_0x03",
      "cid_v0": "QmX9tjYhUY1V7cAgP7URySk5kTn78NbK3YWEwgTieasmQT"
    },
    ...
    {
      "volume_interface_id": "mappings_starting_000_001_000",
      "chapter_interface_id": "signatures_0xff",
      "cid_v0": "QmcMH95qzqaNvgAqFCm5RZ8Z1HwFRy2sMdMHFsb627yzR8"
    }
  ]
}
```
# Databases

The following are descriptions of different databases that have been
implemented in min-know to perform transformation into the TODD-compliant form.

- [Databases](#databases)
    - [Address appearances index](#address-appearances-index)
    - [Nametags](#nametags)


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
    - By address starting characters (0x00 - 0xff), which equates to 256 cChapters per Volume.
- How often should `Volumes` be pulbished (define a `Volume` cadence)
    - Every 10,000 new address additions (E.g., there would be ~72 editions to date).
    - This includes appending new nametags to addresses already in the database.

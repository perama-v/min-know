# Examples

This library helps with the [address-appearance-index][1]. See the individual
example `.rs` files for how to use the library, or the library docs for a more
general overview.

## Usage

There are two main use cases:

- User mode
- Maintainer mode

[1]: https://github.com/perama-v/address-appearance-index-specs

### User mode

End user of the address-appearance-index. Someone with an address who wants to
find out which transactions their address appears in.

```sh
cargo run --example user_0_find_transactions
```

If you run a local node, you can use that (pretend it is a Portal node)
for the following examples:
```sh
cargo run --example user_1_transaction_receipt
cargo run --example user_2_inspect_transaction_logs
```
This example showcases the end goal, but invovles calls to external APIs.
```
cargo run --example user_3_decode_via_apis
```
### Maintainer mode

Creating, updating and providing the address-appearance-index.

For example:
- A portal network bridge client

The examples process many transactions and are best run with `--release` tag.

```sh
cargo run --release --example maintainer_create_index
cargo run --release --example maintainer_extend_index
cargo run --release --example maintainer_audit_correctness
```
## Setup

This library can be used with sample data or real data.

### Sample data

Use 4 of the ~3000 Unchained Index chunk files to test this library
using only ~200MB total.

First get sample data using this example, which looks for the chunks
locally then tries to download them from the github for this library.

Samples may be derived with this command, so best to run with `--release` tag.

```sh
cargo run --release --example get_sample_data
```

The chunks can then be used to test the ability of the library to
create the index (derived from the sample chunks) and manifest.

```sh
cargo run --example maintainer_create_index
```

The sample index is now ready for use. Then manifest can be checked:

```sh
cargo run --example user_check_completeness
```

The transaction information for an address can be found:

```sh
cargo run --example user_0_find_transactions
```

### Real data

To use this library on real data, the address-appearance-index must
be obtained. This can be done in three ways, depending on what
resources you have.

- Construct the Unchained Index using an archive "trace_" enabled node and
trueblocks-core then use this library to construct the address-appearance-index.
- Download the Unchaindex Index from IPFS then use this library to construct the address-appearance-index.
- Obtain the address-appearance-index from a peer(s) who has it already.
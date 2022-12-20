# Examples

The examples (`./example/*.rs`) here are for interacting with the min-know library.

They fall under the following categories:

- [Examples](#examples)
  - [get_sample_data.rs](#get_sample_datars)
  - [user_*.rs](#user_rs)
  - [maintainter_*.rs](#maintainter_rs)
  - [wallet_*.rs](#wallet_rs)


## get_sample_data.rs

Get sample data to be able to run the examples.

The example processes many transactions and are best run with `--release` tag.
```sh
cargo run --release --example get_sample_data
```

## user_*.rs

Use the sample data from the perspective of an end user of a TODD-compliant database.


End user of the address-appearance-index. Someone with an address who wants to
find out which transactions their address appears in.

```sh
cargo run --example user_1_obtain_part_of_db
cargo run --example user_2_check_local_db
cargo run --example user_3_query_db
```

## maintainter_*.rs

Use the library to create an maintain a TODD-compliant database.

The examples process many transactions and are best run with `--release` tag.
```sh
cargo run --release --example maintainer_create_db
cargo run --release --example maintainer_extend_db
cargo run --release --example maintainer_generate_manifest
cargo run --release --example maintainer_repair_index
```

## wallet_*.rs

Use the sample data from the perspective of a Portal Node user who is trying
to get information about their wallet activity without using APIs.

This involves a TODD-compliant address appearance index and identifies
where APIs could be replaced with other TODD-compliant databases.

If you run a local node, you can use that (pretend it is a Portal node)
for the following examples:
```sh
cargo run --example wallet_1_transaction_receipt
cargo run --example wallet_2_inspect_transaction_logs
cargo run --example wallet_3_decode_via_apis
```

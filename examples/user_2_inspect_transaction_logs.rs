use std::env;

use min_know::{
    contract_utils::metadata::ipfs_cid_from_runtime_bytecode,
    types::{AddressIndexPath, Network},
    IndexConfig,
};
use anyhow::anyhow;
use web3::types::BlockNumber;

/// Uses index data and a theoretical local Ethereum portal node to
/// decode information for a user.
///
/// A transaction is inspected for logs, which contain event
/// signatures and the contract from which they were emitted.
///
/// Additionally, the contract code can be inspected and the metadata
/// extracted, which may contain a link to the contract ABI.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    //let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";

    // Random addresses picked from block in sample range.
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9"; // 2

    //let address = "0xcb776c47291b55bf02b159810712f6897874f1cc"; // 7
    //let address = "0x691e27c4c24cf8a5700563e42dadf66b557f372c"; // 44
    //let address = "0x00d83bf7cec1f97489cf324aa8d159bae6aa4df5"; // 1
    //let address = "0xebfd902f83d8ec838ad24259b5bf9617e1b774fc"; // 1
    //let address = "0x029f388ac4d5c8bff490550ce0853221030e822b"; // 339
    //let address = "0xae32371368e500c01068f4fe444aa3cedb48fab4"; // 1
    //let address = "0x00bdb5699745f5b860228c8f939abf1b9ae374ed"; // 1504
    //let address = "0xbf705e134a86c67b703a601c8d5a6caab06cbfd0"; // 7

    let data_dir = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&data_dir, &network);
    // from local index.
    let appearances = index.find_transactions(address)?;

    let portal_node = "http://localhost:8545";
    let transport = web3::transports::Http::new(portal_node)?;
    let web3 = web3::Web3::new(transport);

    let Some(tx) = appearances.get(0)
        else {return Err(anyhow!("No data for this transaction id."))};

    // portal node eth_getTransactionByBlockNumberAndIndex
    let tx_data = web3
        .eth()
        .transaction(tx.as_web3_tx_id())
        .await?
        .ok_or_else(|| anyhow!("No data for this transaction id."))?;
    // portal node eth_getTransactionReceipt
    let tx_receipt = web3
        .eth()
        .transaction_receipt(tx_data.hash)
        .await?
        .ok_or_else(|| anyhow!("No receipt for this transaction hash."))?;

    println!(
        "Tx {:?} has {:#?} logs:\n",
        tx_receipt.transaction_hash,
        tx_receipt.logs.len()
    );

    for log in tx_receipt.logs {
        println!(
            "Contract: {:?}\n\tTopic logged: {:?}",
            log.address,
            log.topics.get(0).unwrap(),
        );
        // portal node eth_getCode
        let code = web3
            .eth()
            .code(log.address, Some(BlockNumber::Latest))
            .await?
            .0;

        match ipfs_cid_from_runtime_bytecode(code.as_ref()) {
            Ok(None) => {}
            Ok(cid) => {
                println!("\tIPFS metadata CID: {:?}", cid.unwrap());
            }
            Err(e) => return Err(e),
        };
    }
    /*
    Tx 0x1a8d94dda1694bad33384215bb3dc0a56652b7069c71d2b1afed35b24c9b54df has 5 logs

    Contract: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
            Topic logged: 0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c
    Contract: 0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2
            Topic logged: 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
    Contract: 0x106d3c66d22d2dd0446df23d7f5960752994d600
            Topic logged: 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
            IPFS metadata CID: "QmZwxURkw5nD5ZCnrhqLdDFG1G52JYKXoXhvvQV2e6cmMH"
    Contract: 0x1636a5dfcf7a21945c06d1bea40b52ce975ea614
            Topic logged: 0x1c411e9a96e071241c2f21f7726b17ae89e3cab4c78be50e062b03a9fffbbad1
    Contract: 0x1636a5dfcf7a21945c06d1bea40b52ce975ea614
            Topic logged: 0xd78ad95fa46c994b6551d0da85fc275fe613ce37657fb8d5e3d130840159d822
    */

    Ok(())
}

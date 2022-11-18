use std::env;

use anyhow::anyhow;
use min_know::{
    types::{AddressIndexPath, Network},
    IndexConfig,
};
use web3::types::H256;

/// Uses index data and a theoretical local Ethereum portal node to
/// decode information for a user.
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9";
    let data_dir = AddressIndexPath::Sample;
    let network = Network::default();
    let index = IndexConfig::new(&data_dir, &network);
    let appearances = index.find_transactions(address)?;
    println!("Level 1 complete: User transactions found.\n");
    // Suppose that the user was running a lightweight portal client
    // https://github.com/ethereum/portal-network-specs#the-json-rpc-api
    // They could use the eth_getTransactionByBlockNumberAndIndex
    // to get transactions.

    let portal_node = "http://localhost:8545";
    let transport = web3::transports::Http::new(portal_node)?;
    let web3 = web3::Web3::new(transport);

    let mut single_tx_hash = H256::default();
    for (i, tx) in appearances.iter().enumerate() {
        if i > 5 {
            break;
        }

        // eth_getTransactionByBlockNumberAndIndex
        let tx_data = web3
            .eth()
            .transaction(tx.as_web3_tx_id())
            .await?
            .ok_or_else(|| anyhow!("No data for this transaction id."))?;

        println!("\nSender: {:?}", tx_data.from);
        println!("Nonce: {}", tx_data.nonce);
        println!("Recipient: {:?}", tx_data.to);
        println!("Gas price: {:?}", tx_data.gas_price);
        println!("Number of bytes passed in: {:?}", tx_data.input.0.len());

        if i == 0 {
            single_tx_hash = tx_data.hash;
        }
    }
    println!("Level 2 complete: User transaction ids retrieved.\n");

    // Pick a single tx and use its newly acquired tx hash to get logs.
    let tx_receipt = web3
        .eth()
        .transaction_receipt(single_tx_hash)
        .await?
        .ok_or_else(|| anyhow!("No receipt for this transaction hash."))?;

    println!("Transaction gas used: {:?}", tx_receipt.gas_used);
    println!("Transaction logs: {:#?}", tx_receipt.logs);

    println!("Level 3 complete: Transaction logs retrieved.\n");
    Ok(())
}

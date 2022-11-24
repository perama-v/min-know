use std::{env, str::FromStr};

use anyhow::{anyhow, bail, Result};
use eip55::checksum;
use reqwest::{header::CONTENT_TYPE, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web3::types::{BlockNumber, H160, H256};

use min_know::{
    contract_utils::metadata::cid_from_runtime_bytecode,
    types::{AddressIndexPath, Network},
    IndexConfig,
};

const FOURBYTE: &str = "https://www.4byte.directory/api/v1/event-signatures/";
const SOURCIFY_FULL: &str = "https://repo.sourcify.dev/contracts/full_match/1/";
const SOURCIFY_PARTIAL: &str = "https://repo.sourcify.dev/contracts/partial_match/1/";

/// Uses combination of external APIs, local index data and a
/// theoretical local Ethereum portal node to
/// decode information for a user.
///
/// ## External data sources
///
/// - Contract ABI is pulled from https://www.sourcify.dev
/// - Event signatures are pulled from https://4byte.directory
///
/// IPFS would ideally replace these sources, not done here to proceed with
/// proof of concept.
///
/// Some ideas for both would be to have sourcify and 4byte both publish
/// annual immutable "editions" where volumes of their data could
/// be downloaded and pinned more readily, without CIDs changing. This
/// might improve data availability on IPFS by allowing more participants.
#[tokio::main]
async fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");

    //let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";

    // Random addresses picked from block in sample range.
    let address = "0x846be97d3bf1e3865f3caf55d749864d39e54cb9"; // 2

    // let address = "0xcb776c47291b55bf02b159810712f6897874f1cc"; // 7
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
    let appearances = index.find_transactions(address)?;
    println!(
        "(sample index data) Address {} appeared in {} transactions",
        &address,
        appearances.len()
    );

    let portal_node = "http://localhost:8545";
    let transport = web3::transports::Http::new(portal_node)?;
    let web3 = web3::Web3::new(transport);

    let tx = appearances
        .get(0)
        .ok_or_else(|| anyhow!("No data for this transaction id."))?;
    // eth_getTransactionByBlockNumberAndIndex
    let tx_data = web3
        .eth()
        .transaction(tx.as_web3_tx_id())
        .await?
        .ok_or_else(|| anyhow!("No data for this transaction id."))?;
    // eth_getTransactionReceipt
    let tx_receipt = web3
        .eth()
        .transaction_receipt(tx_data.hash)
        .await?
        .ok_or_else(|| anyhow!("No receipt for this transaction hash."))?;
    println!(
        "Examining the first transaction  (Tx {:?}) using local node. {:#?} logs found.\n",
        tx_receipt.transaction_hash,
        tx_receipt.logs.len()
    );

    for (index, log) in tx_receipt.logs.iter().enumerate() {
        println!("Log {}, associated with contract: {:?}", index, log.address);
        let topic = log.topics.get(0).unwrap();
        let event_name = method_from_fourbyte_api(topic).await?;

        // Call 4byte registry for event signatures.
        println!(
            "\tTopic {:?}, signature {:?} decoded using 4byte.directory",
            event_name, topic
        );

        // portal node eth_getCode
        let code = web3
            .eth()
            .code(log.address, Some(BlockNumber::Latest))
            .await?
            .0;

        let Ok(maybe_cid) = cid_from_runtime_bytecode(code.as_ref())
            else {return Err(anyhow!("Trouble getting cid from bytecode."))};

        // Later can instead fetch ABI from IPFS.
        match maybe_cid {
            Some(cid) => {
                println!(
                    "\tA CID for contract metadata was in bytecode metadata: {:#?}",
                    cid
                );
            }
            None => {}
        }

        // Call Sourcify API for contract ABIs
        match abi_from_sourcify_api(&log.address).await? {
            Some(abi) => println!("\tContract ABI was obtained from Sourcify:\n\t\t{}", abi),
            None => println!(
                "No matches for ABI were found for address: {}",
                &log.address
            ),
        }
    }
    println!("Filter out unused function names.");
    // Now that that works, one could try to retrieve portions of the 4byte and Sourcify
    // databases then record_key them locally.
    Ok(())
}

/// Returns the first match from 4byte api for an event/topic hash.
///
/// Example endpoint:
///
/// https://www.4byte.directory/api/v1/event-signatures/?hex_signature=0xe1fffcc4
///
/// ## Hash collisions
/// Each decoded candidate response is hashed and compared to the full 32 byte signature
/// (present in the transaction log).
pub async fn method_from_fourbyte_api(topic: &H256) -> Result<Option<String>> {
    let sig = &topic.0[0..4];
    let hex_sig = format!("0x{}", hex::encode(sig));
    let url = Url::from_str(FOURBYTE)?;
    let client = reqwest::Client::new();
    let response: FourBytePage = client
        .get(url)
        .query(&[("hex_signature", hex_sig)])
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?
        .json()
        .await?;
    // Hash to check each decoded response.
    for r in response.results {
        let target = hex::encode(&topic);
        let candidate_full_hash = r.hex_signature.trim_start_matches("0x");
        if candidate_full_hash == target {
            return Ok(Some(r.text_signature));
        }
    }
    return Ok(None);
}

/// Returns the sourcify url target for a given contract address.
pub async fn abi_from_sourcify_api(address: &H160) -> Result<Option<String>> {
    let client = reqwest::Client::new();
    let a = format!("{}/{}", as_checksummed(address), "metadata.json");

    let url = Url::from_str(SOURCIFY_FULL)?.join(&a)?;
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await;
    let Ok(r) = response else {bail!("The request failed for {}", a)};
    match r.status() {
        StatusCode::OK => {
            let v: Value = r.json().await?;
            let contract_summary = summary_of_abi_from_json(v).unwrap();
            return Ok(Some(contract_summary));
        }
        // May not have a full match, so for any error, continue on.
        _ => {
            // println!("Status code: {} for full match. Will try for partial match", r.status());
        }
    }

    // May not match on full
    let url = Url::from_str(SOURCIFY_PARTIAL)?.join(&a)?;
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await;
    let Ok(r) = response else {bail!("The request failed for {}", a)};
    match r.status() {
        StatusCode::OK => {
            let v: Value = r.json().await?;
            let contract_summary = summary_of_abi_from_json(v).unwrap();
            return Ok(Some(contract_summary));
        }
        _ => {
            // println!("Status code: {} for request for partial match", r.status());
            Ok(None)
        }
    }
}

/// Takes a web3.rs address and returns checksummed String.
///
/// E.g., "0xabCd...1234"
fn as_checksummed(address: &H160) -> String {
    let s = h160_to_string(address);
    checksum(&s)
}

/// Converts H160 to String.
fn h160_to_string(address: &H160) -> String {
    //format!("0x{:0>20}", hex::encode(address))
    hex::encode(address)
}

/// Converts String to H160.
fn string_to_h160(address: &str) -> Result<H160> {
    let vector = hex::decode(address.trim_start_matches("0x"))?;
    let tried: Result<[u8; 20], _> = vector.try_into();
    let array = match tried {
        Ok(a) => a,
        Err(e) => return Err(anyhow!("Couldn't byte vector convert address: {:?}", e)),
    };
    Ok(H160(array))
}

#[test]
fn address_conversions() {
    let input = "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
    let address = string_to_h160(input).unwrap();
    let output = h160_to_string(&address);
    assert_eq!(input, output);
}

#[derive(Serialize, Deserialize, Debug)]
/// Response for a match query on event signatures at 4byte.directory.
pub struct FourBytePage {
    next: Option<String>,
    previous: Option<u32>,
    count: Option<u32>,
    results: Vec<FourByteResponse>,
}

#[derive(Serialize, Deserialize, Debug)]
/// Content for a single match at 4byte.directory
pub struct FourByteResponse {
    id: u32,
    created_at: String,
    text_signature: String,
    hex_signature: String,
    bytes_signature: String,
}

#[test]
fn parse_metadata() {
    let metadata_str = r#"
    {"compiler":{"version":"0.4.19+commit.c4cbbb05"},"language":"Solidity","output":{"abi":[{"constant":true,"inputs":[],"name":"name","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"guy","type":"address"},{"name":"wad","type":"uint256"}],"name":"approve","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"src","type":"address"},{"name":"dst","type":"address"},{"name":"wad","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"wad","type":"uint256"}],"name":"withdraw","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"","type":"address"}],"name":"balanceOf","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"dst","type":"address"},{"name":"wad","type":"uint256"}],"name":"transfer","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[],"name":"deposit","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"constant":true,"inputs":[{"name":"","type":"address"},{"name":"","type":"address"}],"name":"allowance","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"payable":true,"stateMutability":"payable","type":"fallback"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":true,"name":"guy","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Approval","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":true,"name":"dst","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Transfer","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"dst","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Deposit","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"src","type":"address"},{"indexed":false,"name":"wad","type":"uint256"}],"name":"Withdrawal","type":"event"}],"devdoc":{"methods":{}},"userdoc":{"methods":{}}},"settings":{"compilationTarget":{"WETH9.sol":"WETH9"},"libraries":{},"optimizer":{"enabled":false,"runs":200},"remappings":[]},"sources":{"WETH9.sol":{"keccak256":"0x4f98b4d0620142d8bea339d134eecd64cbd578b042cf6bc88cb3f23a13a4c893","urls":["bzzr://8f5718790b18ad332003e9f8386333ce182399563925546c3130699d4932de3e"]}},"version":1
    }"#;
    let metadata_json: Value = serde_json::from_str(metadata_str).unwrap();
    let summary = summary_of_abi_from_json(metadata_json).unwrap();
    println!("Summary: {}", summary);
}

/// Gets a human readable summary of contract metadata.
///
/// Parses a JSON string representing contract metadata and returns name of contract and
/// information about functions as a printable string.
fn summary_of_abi_from_json(metadata: Value) -> Result<String> {
    let contract_name = &metadata["settings"]["compilationTarget"];
    let mut summary = format!("Contract: {}", contract_name);
    let n_funcs = match &metadata["output"]["abi"] {
        Value::Array(a) => a.len(),
        _ => 0,
    };
    for n in 0..n_funcs {
        let loc = format!("/output/abi/{}", n);
        let func = metadata
            .pointer(&loc)
            .ok_or_else(|| anyhow!("Could not read abi from json at loc: {}", &loc))?;
        let f = format!(
            "\n\t{} {} {}.\n\t\tInputs: {}\n\t\tOutputs: {}",
            &func["type"],
            &func["stateMutability"],
            &func["name"],
            &func["inputs"],
            &func["outputs"]
        );
        summary.push_str(&f);
    }
    Ok(summary)
}

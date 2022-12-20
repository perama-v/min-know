//! This module helps with parsing information received from a node.
//!
//! For example
//! contract runtime bytecode contains source code metadata that can be decoded.
use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bs58;
use cbor::{Cbor, Decoder};

/// Returns the IPFS CID extracted from the on-chain runtime bytecode of a
/// contract.
///
/// For more information, see:
/// - https://docs.sourcify.dev/blog/verify-contracts-perfectly/
/// - https://docs.soliditylang.org/en/latest/metadata.html
///
/// Note that other resources are available inside the metadata, such as
/// the swarm hash (not currently fetched).
pub fn cid_from_runtime_bytecode(runtime_bytecode: &[u8]) -> Result<Option<MetadataSource>> {
    let metadata = read_metadata(runtime_bytecode)?;
    cid_from_metadata(metadata)
}

#[derive(Debug, PartialEq)]
pub enum MetadataSource {
    Ipfs(String),
    Swarm(String),
}

/// Decodes the IPFS CID from the CBOR-encoded metadata bytes.
///
/// The runtime bytecode must first have the contract code and metadata-length bytes
/// removed prior to being passed here.
fn cid_from_metadata(metadata: &[u8]) -> Result<Option<MetadataSource>> {
    let mut d = Decoder::from_bytes(metadata);
    let cbor = d
        .items()
        .next()
        .ok_or_else(|| anyhow!("Couldn't decode contract metadata CBOR."))??;
    match cbor {
        Cbor::Map(m) => determine_source(m),
        _ => Ok(None),
    }
}

/// Looks for known sources (Swarm, IFPS) in the CBOR hashmap.
///
/// Known keys have the following form: "ipfs" or "bzzr0" or "bzzr1".
fn determine_source(m: HashMap<String, Cbor>) -> Result<Option<MetadataSource>> {
    for key in m.keys() {
        let Some(source) = m.get(key) else { continue };
        // Key exists
        if key.starts_with("ipfs") {
            if let Cbor::Bytes(b) = source {
                let bytes = &b.0;
                let cid = bs58::encode(bytes).into_string();
                return Ok(Some(MetadataSource::Ipfs(cid)));
            }
        }

        if key.starts_with("bzz") {
            if let Cbor::Bytes(b) = source {
                let bytes = &b.0;
                let cid = hex::encode(bytes);
                return Ok(Some(MetadataSource::Swarm(cid)));
            }
        }
    }
    Ok(None)
}

#[test]
fn cid_extraction_1() {
    let test_metadata = "a2646970667358221220c019e4614043d8adc295c3046ba5142c603ab309adeef171f330c51c38f1498964736f6c6343000804";
    let bytes = hex::decode(test_metadata).unwrap();
    let cid = cid_from_metadata(&bytes).unwrap().unwrap();
    let expected = MetadataSource::Ipfs(String::from(
        "QmbGXtNqvZYEcbjK6xELyBQGEmzqXPDqyJNoQYjJPrST9S",
    ));
    assert_eq!(cid, expected);
}

#[test]
fn cid_extraction_2() {
    let test_metadata =
        "a165627a7a72305820deb4c2ccab3c2fdca32ab3f46728389c2fe2c165d5fafa07661e4e004f6c344a";
    let bytes = hex::decode(test_metadata).unwrap();
    let cid = cid_from_metadata(&bytes).unwrap().unwrap();
    let expected = MetadataSource::Swarm(String::from(
        "deb4c2ccab3c2fdca32ab3f46728389c2fe2c165d5fafa07661e4e004f6c344a",
    ));
    assert_eq!(cid, expected);
}

/// Pulls the contract metadata from runtime bytecode.
///
/// Uses the final 2 bytes as the length of the metadata.
///
/// input: <runtime bytecode><metadata><metadata length>
///
/// output: <metadata>
fn read_metadata(code: &[u8]) -> Result<&[u8]> {
    let suffix_len = 2;
    let code_len = code.len();
    let metadata_len_bytes = &code[(code_len - suffix_len)..(code_len)];
    let len_as_slice: [u8; 2] = metadata_len_bytes.try_into()?;
    let metadata_len = u16::from_be_bytes(len_as_slice) as usize;
    let start = code_len - suffix_len - metadata_len;
    let end = code_len - suffix_len;
    let m = &code[start..end];
    Ok(m)
}

#[test]
fn trims_metadata_1() {
    let getcode_result = "608060405234801561001057600080fd5b5061012f806100206000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c80632e64cec11460375780636057361d146051575b600080fd5b603d6069565b6040516048919060c2565b60405180910390f35b6067600480360381019060639190608f565b6072565b005b60008054905090565b8060008190555050565b60008135905060898160e5565b92915050565b60006020828403121560a057600080fd5b600060ac84828501607c565b91505092915050565b60bc8160db565b82525050565b600060208201905060d5600083018460b5565b92915050565b6000819050919050565b60ec8160db565b811460f657600080fd5b5056fea2646970667358221220c019e4614043d8adc295c3046ba5142c603ab309adeef171f330c51c38f1498964736f6c63430008040033";
    let bytes = hex::decode(getcode_result).unwrap();
    let trimmed = read_metadata(bytes.as_ref()).unwrap();
    let expected = "a2646970667358221220c019e4614043d8adc295c3046ba5142c603ab309adeef171f330c51c38f1498964736f6c6343000804";
    assert_eq!(trimmed, hex::decode(expected).unwrap());
}

#[test]
fn trims_metadata_2() {
    let getcode_result = "6060604052600436106100af576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806306fdde03146100b9578063095ea7b31461014757806318160ddd146101a157806323b872dd146101ca5780632e1a7d4d14610243578063313ce5671461026657806370a082311461029557806395d89b41146102e2578063a9059cbb14610370578063d0e30db0146103ca578063dd62ed3e146103d4575b6100b7610440565b005b34156100c457600080fd5b6100cc6104dd565b6040518080602001828103825283818151815260200191508051906020019080838360005b8381101561010c5780820151818401526020810190506100f1565b50505050905090810190601f1680156101395780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b341561015257600080fd5b610187600480803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803590602001909190505061057b565b604051808215151515815260200191505060405180910390f35b34156101ac57600080fd5b6101b461066d565b6040518082815260200191505060405180910390f35b34156101d557600080fd5b610229600480803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803590602001909190505061068c565b604051808215151515815260200191505060405180910390f35b341561024e57600080fd5b61026460048080359060200190919050506109d9565b005b341561027157600080fd5b610279610b05565b604051808260ff1660ff16815260200191505060405180910390f35b34156102a057600080fd5b6102cc600480803573ffffffffffffffffffffffffffffffffffffffff16906020019091905050610b18565b6040518082815260200191505060405180910390f35b34156102ed57600080fd5b6102f5610b30565b6040518080602001828103825283818151815260200191508051906020019080838360005b8381101561033557808201518184015260208101905061031a565b50505050905090810190601f1680156103625780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b341561037b57600080fd5b6103b0600480803573ffffffffffffffffffffffffffffffffffffffff16906020019091908035906020019091905050610bce565b604051808215151515815260200191505060405180910390f35b6103d2610440565b005b34156103df57600080fd5b61042a600480803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803573ffffffffffffffffffffffffffffffffffffffff16906020019091905050610be3565b6040518082815260200191505060405180910390f35b34600360003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055503373ffffffffffffffffffffffffffffffffffffffff167fe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c346040518082815260200191505060405180910390a2565b60008054600181600116156101000203166002900480601f0160208091040260200160405190810160405280929190818152602001828054600181600116156101000203166002900480156105735780601f1061054857610100808354040283529160200191610573565b820191906000526020600020905b81548152906001019060200180831161055657829003601f168201915b505050505081565b600081600460003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508273ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925846040518082815260200191505060405180910390a36001905092915050565b60003073ffffffffffffffffffffffffffffffffffffffff1631905090565b600081600360008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054101515156106dc57600080fd5b3373ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff16141580156107b457507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff600460008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205414155b156108cf5781600460008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020541015151561084457600080fd5b81600460008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002060003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825403925050819055505b81600360008673ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000206000828254039250508190555081600360008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825401925050819055508273ffffffffffffffffffffffffffffffffffffffff168473ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef846040518082815260200191505060405180910390a3600190509392505050565b80600360003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205410151515610a2757600080fd5b80600360003373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600082825403925050819055503373ffffffffffffffffffffffffffffffffffffffff166108fc829081150290604051600060405180830381858888f193505050501515610ab457600080fd5b3373ffffffffffffffffffffffffffffffffffffffff167f7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65826040518082815260200191505060405180910390a250565b600260009054906101000a900460ff1681565b60036020528060005260406000206000915090505481565b60018054600181600116156101000203166002900480601f016020809104026020016040519081016040528092919081815260200182805460018160011615610100020316600290048015610bc65780601f10610b9b57610100808354040283529160200191610bc6565b820191906000526020600020905b815481529060010190602001808311610ba957829003601f168201915b505050505081565b6000610bdb33848461068c565b905092915050565b60046020528160005260406000206020528060005260406000206000915091505054815600a165627a7a72305820deb4c2ccab3c2fdca32ab3f46728389c2fe2c165d5fafa07661e4e004f6c344a0029";
    let bytes = hex::decode(getcode_result).unwrap();
    let trimmed = read_metadata(bytes.as_ref()).unwrap();
    let expected =
        "a165627a7a72305820deb4c2ccab3c2fdca32ab3f46728389c2fe2c165d5fafa07661e4e004f6c344a";
    assert_eq!(trimmed, hex::decode(expected).unwrap());
}

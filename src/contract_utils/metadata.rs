use anyhow::anyhow;
use bs58;
use cbor::Decoder;

/// Returns the IPFS CID extracted from the on-chain runtime bytecode of a
/// contract.
///
/// For more information, see:
/// - https://docs.sourcify.dev/blog/verify-contracts-perfectly/
/// - https://docs.soliditylang.org/en/latest/metadata.html
///
/// Note that other resources are available inside the metadata, such as
/// the swarm hash (not currently fetched).
pub fn ipfs_cid_from_runtime_bytecode(
    runtime_bytecode: &[u8],
) -> Result<Option<String>, anyhow::Error> {
    let metadata = read_metadata(runtime_bytecode)?;
    ipfs_cid_from_metadata(metadata)
}

/// Decodes the IPFS CID from the CBOR-encoded metadata bytes.
///
/// The runtime bytecode must first have the contract conde and metadata-length bytes
/// removed prior to being passed here.
fn ipfs_cid_from_metadata(metadata: &[u8]) -> Result<Option<String>, anyhow::Error> {
    let mut d = Decoder::from_bytes(metadata);
    let cbor = d
        .items()
        .next()
        .ok_or_else(|| anyhow!("Couldn't decode contract metadata CBOR."))??;
    match cbor {
        cbor::Cbor::Map(m) => {
            let ipfs = m.get("ipfs");
            match ipfs {
                Some(cbor::Cbor::Bytes(b)) => {
                    let bytes = &b.0;
                    let cid = bs58::encode(bytes).into_string();
                    Ok(Some(cid))
                }
                _ => return Ok(None),
            }
        }
        _ => return Ok(None),
    }
}

#[test]
fn cid_extraction() {
    let test_metadata = "a2646970667358221220c019e4614043d8adc295c3046ba5142c603ab309adeef171f330c51c38f1498964736f6c6343000804";
    let bytes = hex::decode(test_metadata).unwrap();
    let cid = ipfs_cid_from_metadata(&bytes).unwrap();
    assert_eq!(
        cid,
        Some(String::from(
            "QmbGXtNqvZYEcbjK6xELyBQGEmzqXPDqyJNoQYjJPrST9S"
        ))
    );
}

/// Pulls the contract metadata from runtime bytecode.
///
/// Uses the final 2 bytes as the length of the metadata.
///
/// input: <runtime bytecode><metadata><metadata length>
///
/// output: <metadata>
fn read_metadata(code: &[u8]) -> Result<&[u8], anyhow::Error> {
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
fn trims_metadata() {
    let getcode_result = "608060405234801561001057600080fd5b5061012f806100206000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c80632e64cec11460375780636057361d146051575b600080fd5b603d6069565b6040516048919060c2565b60405180910390f35b6067600480360381019060639190608f565b6072565b005b60008054905090565b8060008190555050565b60008135905060898160e5565b92915050565b60006020828403121560a057600080fd5b600060ac84828501607c565b91505092915050565b60bc8160db565b82525050565b600060208201905060d5600083018460b5565b92915050565b6000819050919050565b60ec8160db565b811460f657600080fd5b5056fea2646970667358221220c019e4614043d8adc295c3046ba5142c603ab309adeef171f330c51c38f1498964736f6c63430008040033";
    let bytes = hex::decode(getcode_result).unwrap();
    let trimmed = read_metadata(bytes.as_ref()).unwrap();
    let expected = "a2646970667358221220c019e4614043d8adc295c3046ba5142c603ab309adeef171f330c51c38f1498964736f6c6343000804";
    assert_eq!(trimmed, hex::decode(expected).unwrap());
}

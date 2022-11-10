//! IPFS-related helpers like CID computation.
use cid::multihash::{Code, MultihashDigest};
use cid::{Cid, CidGeneric};
use std::convert::TryFrom;
use std::str::from_utf8;


/// Computes the CIDv1 for the given bytes.
pub fn cid_v1_from_bytes(bytes: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let h = Code::Sha2_256.digest(bytes);
    const RAW: u64 = 0x55;
    let cid = Cid::new_v1(RAW, h);
    Ok(cid.to_string().as_bytes().to_vec())
}

/// Computes the CIDv0 for the given bytes.
pub fn cid_v0_from_bytes(bytes: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let h = Code::Sha2_256.digest(bytes);
    let cid = Cid::new_v0(h)?;
    Ok(cid.to_string().as_bytes().to_vec())
}

#[test]
fn str_to_cid() {
    let cid = cid_v1_from_bytes("beep boop".as_bytes()).unwrap();
    let string = from_utf8(&cid).unwrap();
    assert_eq!(
        string,
        "bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq"
    );
}
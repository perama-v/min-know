use anyhow::Result;
use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};

/// Computes the CIDv0 for the given bytes, returning as a String.
pub fn cid_v0_string_from_bytes(bytes: &[u8]) -> Result<String> {
    let h = Code::Sha2_256.digest(bytes);
    let cid = Cid::new_v0(h)?;
    Ok(cid.to_string())
}

/// Computes the CIDv1 for the given bytes.
#[allow(dead_code)]
pub fn cid_v1_from_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
    let h = Code::Sha2_256.digest(bytes);
    const RAW: u64 = 0x55;
    let cid = Cid::new_v1(RAW, h);
    Ok(cid.to_string().as_bytes().to_vec())
}

/// Computes the CIDv0 for the given bytes.
#[allow(dead_code)]
pub fn cid_v0_from_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
    let h = Code::Sha2_256.digest(bytes);
    let cid = Cid::new_v0(h)?;
    let vec = cid.to_string().as_bytes().to_vec();
    //assert_eq!(vec.len(), 32);
    Ok(vec)
}

#[test]
fn str_to_cidv0() {
    use std::str::from_utf8;

    let cid = cid_v0_from_bytes("beep boop".as_bytes()).unwrap();
    let string = from_utf8(&cid).unwrap();
    assert_eq!(string, "QmY6LjJ1HExi2TgHshc56ecPdVSNaWrzFbWq9sahHFrNoM");
}

#[test]
fn str_to_cidv1() {
    use std::str::from_utf8;

    let cid = cid_v1_from_bytes("beep boop".as_bytes()).unwrap();
    let string = from_utf8(&cid).unwrap();
    assert_eq!(
        string,
        "bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq"
    );
}

//! IPFS-related helpers like CID computation.
use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use std::convert::TryFrom;

const RAW: u64 = 0x55;

/// Converts content identifier (IPFS) from byte vector.
pub fn cid_from_bytes(bytes: &[u8]) -> Result<String, anyhow::Error> {
    let h = Code::Sha2_256.digest(bytes);

    let cid = Cid::new_v1(RAW, h);

    let data = cid.to_bytes();
    let out = Cid::try_from(data)?;
    assert_eq!(cid, out);
    let cid_string = cid.to_string();
    Ok(cid_string)
}

/// Converts content identifier (IPFS) from byte vector.
pub fn cidv0_from_bytes(bytes: &[u8]) -> Result<String, anyhow::Error> {
    let h = Code::Sha2_256.digest(bytes);
    let cid = Cid::new_v0(h)?;
    let cid_string = cid.to_string();
    Ok(cid_string)
}

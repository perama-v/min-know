//! Responsible for SSZ encoding and Snappy compression.
use anyhow::{anyhow, Result};
use ssz::{Decode, Encode};
use std::io::Read;

/// Perfoms ssz encoding and snappy compression.
pub fn encode_and_compress<T>(structured: T) -> Result<Vec<u8>>
where
    T: Encode,
{
    let ssz_encoded = encode(structured)?;
    let ssz_snappy = compress(ssz_encoded)?;
    Ok(ssz_snappy)
}

/// Performs snappy decompression and ssz decoding.
pub fn decode_and_decompress<T>(ssz_snappy_data: Vec<u8>) -> Result<T>
where
    T: Decode,
{
    let ssz_encoded = decompress(ssz_snappy_data)?;
    let structured_data = decode(ssz_encoded)?;
    Ok(structured_data)
}

/// Perfoms ssz encoding of struct into bytes.
pub fn encode<T>(structured: T) -> Result<Vec<u8>>
where
    T: Encode,
{
    let encoded_data = structured.as_ssz_bytes();
    Ok(encoded_data)
}

/// Performs ssz decoding of bytes into struct.
pub fn decode<T>(ssz_bytes: Vec<u8>) -> Result<T>
where
    T: Decode,
{
    let decoded_res = T::from_ssz_bytes(&ssz_bytes);
    // Change DecodeError to anyhow::Error
    match decoded_res {
        Ok(data) => Ok(data),
        Err(e) => Err(anyhow!(
            "Could not decode the SSZ data. Check that the library
            spec version matches the version in the manifest.  {:?}",
            e
        )),
    }
}

/// Performs snappy compression on bytes.
///
/// Takes ssz bytes, returns ssz_snappy bytes.
pub fn compress(ssz_bytes: Vec<u8>) -> Result<Vec<u8>> {
    /*
    Raw encoder (no frames):
    let mut snap_encoder = snap::raw::Encoder::new();
    let compressed_vec = snap_encoder.compress_vec(ssz_bytes.as_slice())?;
    */
    let mut buffer = vec![];
    snap::read::FrameEncoder::new(ssz_bytes.as_slice()).read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Performs decompression on snappy bytes.
///
/// Takes ssz_snappy bytes, returns ssz bytes.
pub fn decompress(ssz_snappy_bytes: Vec<u8>) -> Result<Vec<u8>> {
    /*
    Raw decoder (no frames):
    let mut snap_decoder = snap::raw::Decoder::new();
    let decompressed_vec = snap_decoder.decompress_vec(ssz_snappy_bytes.as_slice())?;
    */
    let mut buffer = vec![];
    snap::read::FrameDecoder::new(ssz_snappy_bytes.as_slice()).read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[test]
fn encode_decode() -> Result<()> {
    use crate::spec::{
        AddressAppearances, AddressIndexVolumeChapter, AppearanceTx, VolumeIdentifier,
    };
    let data_in = AddressIndexVolumeChapter {
        address_prefix: <_>::from("a3".as_bytes().to_vec()),
        identifier: VolumeIdentifier { oldest_block: 0 },
        addresses: <_>::from(vec![
            AddressAppearances {
                address: <_>::from("0xabcde".as_bytes().to_vec()),
                appearances: <_>::from(vec![
                    AppearanceTx {
                        block: 123,
                        index: 40,
                    },
                    AppearanceTx {
                        block: 5553,
                        index: 120,
                    },
                ]),
            },
            AddressAppearances {
                address: <_>::from("0xffffe".as_bytes().to_vec()),
                appearances: <_>::from(vec![
                    AppearanceTx { block: 3, index: 4 },
                    AppearanceTx {
                        block: 92,
                        index: 10,
                    },
                ]),
            },
        ]),
    };
    let encoded = encode_and_compress(data_in.clone())?;
    let data_out = decode_and_decompress(encoded)?;
    assert_eq!(data_in, data_out);
    Ok(())
}

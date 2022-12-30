//! Address Appearance Index (AAI)
use std::fmt::Display;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use ssz::{Decode, Encode};
use ssz_derive::{Decode, Encode};
use ssz_types::{
    typenum::{U1073741824, U20},
    FixedVector, VariableList,
};
use web3::types::{BlockNumber, BlockId, TransactionId};

use crate::{
    extraction::address_appearance_index::AAIExtractor,
    parameters::address_appearance_index::{
        MaxAddressesPerVolume, NumCommonBytes, BLOCKS_PER_VOLUME, NUM_CHAPTERS,
    },
    samples::address_appearance_index::AAISampleObtainer,
    utils::unchained::types::BlockRange,
};

use super::traits::*;

/// Spec for the Address Appearance Index database.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AAISpec {}

impl DataSpec for AAISpec {
    const NUM_CHAPTERS: usize = NUM_CHAPTERS as usize;

    const MAX_VOLUMES: usize = 1_000_000_000;

    type AssociatedVolumeId = AAIVolumeId;

    type AssociatedChapterId = AAIChapterId;

    type AssociatedChapter = AAIChapter;

    type AssociatedRecord = AAIRecord;

    type AssociatedRecordKey = AAIRecordKey;

    type AssociatedRecordValue = AAIRecordValue;

    type AssociatedExtractor = AAIExtractor;

    type AssociatedSampleObtainer = AAISampleObtainer;

    type AssociatedManifest = AAIManifest;

    fn spec_name() -> SpecId {
        SpecId::AddressAppearanceIndex
    }

    fn spec_version() -> String {
        String::from("0.1.0")
    }

    fn spec_schemas_resource() -> String {
        String::from("https://github.com/perama-v/address-index/tree/main/address_appearance_index")
    }

    fn num_chapters() -> usize {
        Self::NUM_CHAPTERS
    }
    /// Gets the ChapterIds relevant for a key.
    fn record_key_to_chapter_id(
        record_key: &Self::AssociatedRecordKey,
    ) -> Result<Self::AssociatedChapterId> {
        let bytes = record_key.key[0..2].to_vec();
        Ok(AAIChapterId {
            val: <_>::from(bytes),
        })
    }
    // Key is a hex string. Converts it to an ssz vector.
    fn raw_key_as_record_key(key: &str) -> Result<Self::AssociatedRecordKey> {
        let raw_bytes = hex::decode(key.trim_start_matches("0x"))?;
        Ok(AAIRecordKey {
            key: <_>::from(raw_bytes),
        })
    }
}

#[derive(
    Clone, Debug, Default, PartialEq, PartialOrd, Hash, Serialize, Deserialize, Encode, Decode,
)]
pub struct AAIVolumeId {
    pub oldest_block: u32,
}
impl VolumeIdMethods<AAISpec> for AAIVolumeId {
    fn interface_id(&self) -> String {
        let mut name = format!("{:0>9}", self.oldest_block);
        for i in [6, 3] {
            name.insert(i, '_');
        }
        format!("volume_{}", name)
    }
    fn nth_id(n: u32) -> Result<Self> {
        // n=0, id=0
        // n=1, id=100_000
        // n=2, id=200_000
        let oldest_block = n * BLOCKS_PER_VOLUME;
        Ok(AAIVolumeId { oldest_block })
    }

    fn is_nth(&self) -> Result<u32> {
        // id=0, n=0
        // id=100_000, n=1
        // id=200_000, n=2
        Ok(self.oldest_block / BLOCKS_PER_VOLUME)
    }

    fn from_interface_id(interface_id: &str) -> Result<Self> {
        let oldest_block = interface_id
            .trim_start_matches("volume")
            .replace('_', "")
            .parse::<u32>()?;
        Ok(AAIVolumeId { oldest_block })
    }
}
impl AAIVolumeId {
    pub fn to_block_range(&self) -> Result<BlockRange> {
        BlockRange::new(self.oldest_block, BLOCKS_PER_VOLUME - 1 + self.oldest_block)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct AAIChapterId {
    pub val: FixedVector<u8, NumCommonBytes>,
}
impl ChapterIdMethods<AAISpec> for AAIChapterId {
    fn interface_id(&self) -> String {
        let chars = hex::encode(self.val.to_vec());
        format!("chapter_0x{}", chars)
    }
    fn nth_id(n: u32) -> Result<Self> {
        if n as usize >= AAISpec::NUM_CHAPTERS {
            bail!("'n' must be <= NUM_CHAPTERS")
        }
        let byte_vec = vec![n as u8];
        let Ok(fv) = FixedVector::<u8, NumCommonBytes>::new(byte_vec) else {
            bail!("Provided vector is too long for Fixed Vector.")
        };
        Ok(AAIChapterId { val: fv })
    }
    fn from_interface_id(id_string: &str) -> Result<Self> {
        let string = id_string.trim_start_matches("chapter_0x");
        let bytes = hex::decode(string)?;
        Ok(AAIChapterId {
            val: <_>::from(bytes),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct AAIChapter {
    pub chapter_id: AAIChapterId,
    pub volume_id: AAIVolumeId,
    pub records: Vec<AAIRecord>,
}
impl ChapterMethods<AAISpec> for AAIChapter {
    fn get(self) -> Self {
        self
    }

    fn volume_id(&self) -> &AAIVolumeId {
        &self.volume_id
    }

    fn chapter_id(&self) -> &AAIChapterId {
        &self.chapter_id
    }

    fn records(&self) -> &Vec<AAIRecord> {
        &self.records
    }

    fn as_serialized_bytes(&self) -> Vec<u8> {
        self.as_ssz_bytes()
    }
    /// Reads a Chapter from file. Currently reads Relic file structure.
    fn from_file(data: Vec<u8>) -> Result<Self> {
        // Files are ssz encoded.
        let chapter = match AAIChapter::from_ssz_bytes(&data) {
            Ok(c) => c,
            Err(e) => bail!(
                "Could not decode the SSZ data. Check that the library
            spec version matches the version in the manifest.  {:?}",
                e
            ),
        };
        Ok(chapter)
    }
    fn filename(&self) -> String {
        format!(
            "{}_{}.ssz",
            self.volume_id().interface_id(),
            self.chapter_id().interface_id()
        )
    }

    fn new_empty(volume_id: &AAIVolumeId, chapter_id: &AAIChapterId) -> Self {
        AAIChapter {
            chapter_id: chapter_id.clone(),
            volume_id: volume_id.clone(),
            records: vec![],
        }
    }
}

impl AAIChapter {
    /// A helper function that converts the old data chapter (file) structure into the new
    /// one.
    ///
    /// This is used to minimise changes to the transformation/exctraction code during
    /// the move to generics and can replaced eventually.
    pub fn from_relic(data: RelicChapter) -> Self {
        let chapter_id = AAIChapterId {
            val: <_>::from(data.address_prefix.to_vec()),
        };
        let volume_id = AAIVolumeId {
            oldest_block: data.identifier.oldest_block,
        };
        let mut records: Vec<AAIRecord> = vec![];
        for item in data.addresses.iter() {
            let r = AAIRecord {
                key: AAIRecordKey {
                    key: <_>::from(item.address.to_vec()),
                },
                value: AAIRecordValue {
                    value: <_>::from(item.appearances.to_vec()),
                },
            };
            records.push(r)
        }
        AAIChapter {
            chapter_id,
            volume_id,
            records,
        }
    }
}

pub type DefaultBytesPerAddress = U20;
pub type MaxTxsPerVolume = U1073741824;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct AAIRecord {
    pub key: AAIRecordKey,
    pub value: AAIRecordValue,
}
impl RecordMethods<AAISpec> for AAIRecord {
    fn get(&self) -> &Self {
        self
    }

    fn key(&self) -> &AAIRecordKey {
        &self.key
    }

    fn value(&self) -> &AAIRecordValue {
        &self.value
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct AAIRecordKey {
    pub key: FixedVector<u8, DefaultBytesPerAddress>,
}
impl RecordKeyMethods for AAIRecordKey {
    fn get(self) -> Self {
        self
    }
}

/// Equivalent to AddressAppearances. Consists of a single address and some
/// number of transaction identfiers (appearances).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct AAIRecordValue {
    /// The transactions where the address appeared.
    pub value: VariableList<AAIAppearanceTx, MaxTxsPerVolume>,
}
impl RecordValueMethods for AAIRecordValue {
    fn get(self) -> Self {
        self
    }
    /// Return a String representation of the contents of the RecordValue.
    fn as_strings(&self) -> Vec<String> {
        let mut s: Vec<String> = vec![];
        for v in self.value.iter() {
            let v_str = format!("Tx in block: {}, index: {}", v.block, v.index);
            s.push(v_str)
        }
        s
    }
}

/// An identifier for a single transaction.
///
/// Consists of block number and index within that block.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct AAIAppearanceTx {
    /// The Ethereum execution block number.
    pub block: u32,
    /// The index of the transaction in a block.
    pub index: u32,
}

impl AAIAppearanceTx {
    /// Converts to web3.rs transaction type.
    pub fn as_web3_tx_id(&self) -> web3::types::TransactionId {
        let block_num = BlockNumber::Number(<_>::from(self.block));
        let tx_block_id = BlockId::Number(block_num);
        let tx_index = <_>::from(self.index);
        TransactionId::Block(tx_block_id, tx_index)
    }
}

//
//
// Relic structures. The files are currently stored in this format, but this
// can be changed to a simpler format (using RecordKey and RecordValue directly).
//
//

#[derive(PartialEq, Debug, Encode, Decode, Clone)]
pub struct RelicChapter {
    /// Prefix common to all addresses that this data covers.
    pub address_prefix: FixedVector<u8, DefaultBytesPerAddress>,
    /// The blocks that this chunk data covers.
    pub identifier: RelicVolumeIdentifier,
    /// The addresses that appeared in this range and the relevant transactions.
    pub addresses: VariableList<RelicAddressAppearances, MaxAddressesPerVolume>,
}

#[derive(Debug, Default, PartialEq, Clone, Encode, Decode)]
pub struct RelicAddressAppearances {
    /// The address that appeared in a transaction.
    pub address: FixedVector<u8, DefaultBytesPerAddress>,
    /// The transactions where the address appeared.
    pub appearances: VariableList<AAIAppearanceTx, MaxTxsPerVolume>,
}

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize, Deserialize)]
pub struct RelicVolumeIdentifier {
    pub oldest_block: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AAIManifest {
    pub spec_version: String,
    pub schemas: String,
    pub database_interface_id: String,
    pub latest_volume_identifier: String,
    pub chapter_cids: Vec<AAIManifestChapter>,
}

impl ManifestMethods<AAISpec> for AAIManifest {
    fn spec_version(&self) -> &str {
        &self.spec_version
    }

    fn set_spec_version(&mut self, version: String) {
        self.spec_version = version
    }

    fn schemas(&self) -> &str {
        &self.schemas
    }

    fn set_schemas(&mut self, schemas: String) {
        self.schemas = schemas
    }

    fn database_interface_id(&self) -> &str {
        &self.database_interface_id
    }

    fn set_database_interface_id(&mut self, id: String) {
        self.database_interface_id = id;
    }

    fn latest_volume_identifier(&self) -> &str {
        &self.latest_volume_identifier
    }

    fn set_latest_volume_identifier(&mut self, volume_interface_id: String) {
        self.latest_volume_identifier = volume_interface_id
    }

    fn cids(&self) -> Result<Vec<ManifestCids<AAISpec>>> {
        let mut result: Vec<ManifestCids<AAISpec>> = vec![];
        for chapter in &self.chapter_cids {
            let volume_id = AAIVolumeId::from_interface_id(&chapter.volume_interface_id)?;
            let chapter_id = AAIChapterId::from_interface_id(&chapter.chapter_interface_id)?;
            result.push(ManifestCids {
                cid: chapter.cid_v0.clone(),
                volume_id,
                chapter_id,
            })
        }
        Ok(result)
    }

    fn set_cids<U: AsRef<str> + Display>(&mut self, cids: &[(U, AAIVolumeId, AAIChapterId)]) {
        for (cid, volume_id, chapter_id) in cids {
            let chapter = AAIManifestChapter {
                volume_interface_id: volume_id.interface_id(),
                chapter_interface_id: chapter_id.interface_id(),
                cid_v0: cid.to_string(),
            };
            self.chapter_cids.push(chapter)
        }
        // Sort by VolumeId, then by ChapterId for ties.
        self.chapter_cids.sort_by(|a, b| {
            a.volume_interface_id
                .cmp(&b.volume_interface_id)
                .then(a.chapter_interface_id.cmp(&b.chapter_interface_id))
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AAIManifestChapter {
    pub volume_interface_id: String,
    pub chapter_interface_id: String,
    pub cid_v0: String,
}

#[test]
fn encode_decode() -> Result<()> {
    use crate::specs::address_appearance_index::AAIAppearanceTx;
    let data_in = AAIAppearanceTx {
        block: 122455,
        index: 23,
    };
    let encoded = data_in.clone().as_ssz_bytes();
    let data_out: AAIAppearanceTx = <_>::from_ssz_bytes(&encoded).unwrap();
    assert_eq!(data_in, data_out);
    Ok(())
}

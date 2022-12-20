use anyhow::{anyhow, Result};
use log::debug;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;

use super::{
    constants::{AD_ENTRY, AP_ENTRY},
    files::{file_structure, get_range, no_unexpected_appearances},
    structure::{AddressData, AddressEntry, Body, Header, TransactionId},
};

/**
Unchained Index .bin file read and tracker.

Is a helper used in the creation of the address-appearance-index.

# Example
The following example reads specific chunk files, filtering by block
and address.

Transaction data meets the following requirements:
- Specific chunk files
- Specific block heights
- Specific addresses

```no_run
use anyhow::bail;
use min_know::{
    config::choices::{DataKind, DirNature},
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
    utils::unchained::{
        files::ChunksDir,
        types::{BlockRange, UnchainedFile},
    },
};
let db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample)?;

let desired_blocks = BlockRange {
    old: 0,
    new: 16_000_000,
};
let chunk_files: ChunksDir = ChunksDir::new(&db.config.raw_source)?;
let Some(relevant_files) = chunk_files.for_range(&desired_blocks) else {
    bail!("No relevant files")};

let address_starts_with = String::from("4e");

// Counter for the appearances that match the description.
let mut sum = 0;
for chunk in relevant_files {
    let path = chunk.path.to_owned();
    let mut file = UnchainedFile::new(path, desired_blocks)?;
    // Read appearances that have correct leading char and are in desired range.
    file.with_parsed(&address_starts_with)?;
    sum += file.parsed.len();
}
// The sample dir has 6204 addresses that start with 0x4e.
assert_eq!(sum, 6204);
# Ok::<(), anyhow::Error>(())
```
*/
pub struct UnchainedFile {
    pub path: PathBuf,
    pub reader: BufReader<File>,
    pub header: Header,
    pub body: Body,
    pub present: BlockRange,
    pub desired: BlockRange,
    pub contains_unwanted_blocks: bool,
    pub parsed: Vec<AddressData>,
}

impl UnchainedFile {
    /// Obtains metadata and prepares Unchained Index file for reading.
    pub fn new(path: PathBuf, desired: BlockRange) -> Result<Self> {
        let file = File::open(&path)?;
        let mut reader: BufReader<File> = BufReader::new(file);
        let header = Header::from_reader(reader.by_ref(), &path)?;
        let body: Body = file_structure(&header);
        let parsed: Vec<AddressData> = vec![AddressData::default()];
        let present: BlockRange = get_range(&path)?;

        // If no intersection, return error
        if !present.intersection_exists(&desired) {
            return Err(anyhow!(
                "File only has appearances ({}-{}) out of desired range ({}-{}).",
                present.old,
                present.new,
                desired.old,
                desired.new,
            ));
        };
        let contains_unwanted_blocks = !present.is_volume_of(&desired);
        Ok(UnchainedFile {
            path,
            reader,
            header,
            body,
            present,
            desired,
            contains_unwanted_blocks,
            parsed,
        })
    }

    /// Populates the self.parsed field with relevant AddressData.
    ///
    /// Only addresses that begin with the specified hex characters are included.
    /// E.g., "0xbe"
    ///
    /// Algorithm:
    /// 1. Iterate over address entries, starting reader at the address table.
    /// 2. For current address entry, read the address, offset and count.
    /// 3. Determine jump location using offset and count.
    /// 4. Jump to the appearance table.
    /// 5. Read and store transactions in vector, looping until count satisfied.
    /// 6. Skip transactions outside desired RANGE.
    /// 7. Save to transactions to database, adding to existing AddressData for that address.
    /// 8. Update address byte index for the next entry
    /// 9. Jump back to address table, go to 2.
    pub fn with_parsed(&mut self, address_leading_char: &str) -> Result<()> {
        let address_starting_bytes = hex::decode(address_leading_char)?;
        let mut txs: Vec<AddressData> = vec![];
        let mut addresses_parsed = 0;
        // 1.
        while addresses_parsed < self.header.n_addresses {
            // 2.
            let address_entry = AddressEntry::from_reader(self.reader.by_ref())?;
            addresses_parsed += 1;
            // 3.
            let app_passed = address_entry.offset as usize * AP_ENTRY;
            self.body.appearances.current = self.body.appearances.start + app_passed;
            let address = address_entry.address.clone();
            if address.starts_with(address_starting_bytes.as_ref()) {
                // 4. to 7.
                let potential_appearances: Option<Vec<TransactionId>> =
                    self.parse_appearances(&address_entry)?;

                if let Some(appearances) = potential_appearances {
                    let appearances_for_address = AddressData {
                        address,
                        appearances,
                    };
                    txs.push(appearances_for_address);
                }
                // (else) All transactions for this address were outside the desired range.
            }
            // 8.
            self.body.addresses.current += AD_ENTRY;
            // 9.
            self.reader
                .seek(SeekFrom::Start(self.body.addresses.current as u64))?;
        }
        self.parsed = txs;

        debug!(
            "In {:?}. {:0>7} addresses started with 0x{} and had tx in range ({}-{}). Chunk attributes: nAddr {:0>7}, nApp {:0>7}.",
            self.path.file_name().unwrap(), self.parsed.len(), address_leading_char,
            self.desired.old, self.desired.new,
            self.header.n_addresses, self.header.n_appearances
        );
        Ok(())
    }

    /// Processes the appearances (transactions) for a given address
    fn parse_appearances(
        &mut self,
        address_entry: &AddressEntry,
    ) -> anyhow::Result<Option<Vec<TransactionId>>> {
        let mut appearances_parsed = 0;
        let mut entries: Vec<TransactionId> = Vec::new();
        while appearances_parsed < address_entry.count {
            // 4.
            self.reader
                .seek(SeekFrom::Start(self.body.appearances.current as u64))?;
            // 5.
            let appearance: TransactionId = TransactionId::from_reader(self.reader.by_ref())?;
            no_unexpected_appearances(&appearance, self)?;
            // 6.
            if self.contains_unwanted_blocks {
                if self.desired.contains(&appearance) {
                    entries.push(appearance);
                } else {
                    // Exclude transactions not within the desired block range.
                }
            } else {
                entries.push(appearance);
            }
            self.body.appearances.current += AP_ENTRY;
            appearances_parsed += 1;
        }
        if entries.is_empty() {
            // Transactions exist in the chunk file, but not within the desired block range.
            // No errors, no transactions.
            return Ok(None);
        }
        Ok(Some(entries))
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BlockRange {
    pub old: u32,
    pub new: u32,
}

impl BlockRange {
    /// New range of blocks.
    pub fn new(old_block_number: u32, new_block_number: u32) -> Result<Self> {
        if old_block_number >= new_block_number {
            return Err(anyhow!(
                "Older block {} must be less than newer block {}.",
                old_block_number,
                new_block_number
            ));
        }
        Ok(BlockRange {
            old: old_block_number,
            new: new_block_number,
        })
    }

    /// True if there are any common blocks for two ranges.
    pub fn intersection_exists(&self, other: &BlockRange) -> bool {
        /*
        Start can't come after end of other.
        End can't come after start of other.End can't comex1 <= y2 && y1 <= x2
        |-----| self
           |----| other
        */
        if self.old <= other.new && other.old <= self.new {
            return true;
        }
        false
    }
    /// True if blocks in a range are all within another range.
    fn is_volume_of(&self, other: &BlockRange) -> bool {
        if self.old >= other.old && self.new <= other.new {
            return true;
        }
        false
    }
    /// True if range contains the specified transaction.
    fn contains(&self, tx: &TransactionId) -> bool {
        if self.old <= tx.block && self.new >= tx.block {
            return true;
        }
        false
    }
}

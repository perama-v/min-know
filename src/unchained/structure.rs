//! Contains the structure of the Unchained Index as defined in
//! the Unchained Index specification.
use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{io::Read, path::PathBuf};

use super::constants::{ADDR, MAGIC, VAL, VER};

#[derive(Default)]
/// Stores values extracted from file header.
pub struct Header {
    pub n_addresses: u32,
    pub n_appearances: u32,
}

impl Header {
    /// Obtains values from file header and validates magic number.
    pub fn from_reader(
        mut rdr: impl Read,
        path: &PathBuf,
    ) -> anyhow::Result<Header, anyhow::Error> {
        let mut magic: [u8; VAL] = [0; VAL];
        rdr.read_exact(&mut magic)?;
        if magic != MAGIC {
            return Err(anyhow!("file {:?} has incorrect magic bytes", path));
        }
        let mut version: [u8; VER] = [0; VER];
        rdr.read_exact(&mut version)?;
        let n_addresses = rdr.read_u32::<LittleEndian>()?;
        let n_appearances = rdr.read_u32::<LittleEndian>()?;
        Ok(Header {
            n_addresses,
            n_appearances,
        })
    }
}

/// Records information about important byte indices in the chunk file.
pub struct Body {
    /// Table in binary file containing addresses.
    pub addresses: Section,
    /// Table in binary file containing appearances (transaction IDs).
    pub appearances: Section,
}

/// Byte indices and length of entry for particular section.
pub struct Section {
    /// Byte index of start of section.
    pub start: usize,
    /// Which byte is currently of interest for this section.
    pub current: usize,
    /// Byte index of end of section.
    pub end: usize,
}

/// Content of an entry in the Addresses table.
#[derive(Clone)]
pub struct AddressEntry {
    /// Address bytes. Length 20 bytes.
    pub address: Vec<u8>,
    pub offset: u32,
    pub count: u32,
}

impl AddressEntry {
    /// Reads an address entry from the current reader position.
    pub fn from_reader(mut rdr: impl Read) -> std::io::Result<Self> {
        let mut addr_buf: [u8; ADDR] = [0; ADDR];
        rdr.read_exact(&mut addr_buf)?;
        let address = addr_buf.to_vec();
        let offset = rdr.read_u32::<LittleEndian>()?;
        let count = rdr.read_u32::<LittleEndian>()?;

        Ok(AddressEntry {
            address,
            offset,
            count,
        })
    }
}

/// Holds selected transactions for a given address.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct AddressData {
    /// The address that appeared in a transaction.
    pub address: Vec<u8>,
    /// The transactions where the address appeared.
    pub appearances: Vec<TransactionId>,
}

/// Content of an entry in the Appearances (transactions) table.
#[derive(Debug, PartialEq, Clone)]
pub struct TransactionId {
    /// The Ethereum execution block number.
    pub block: u32,
    /// The index of the transaction in a block.
    pub index: u32,
}

impl TransactionId {
    /// Reads an appearance (Tx) entry from the current reader position.
    pub fn from_reader(mut rdr: impl Read) -> std::io::Result<Self> {
        let block = rdr.read_u32::<LittleEndian>()?;
        let index = rdr.read_u32::<LittleEndian>()?;
        Ok(TransactionId { block, index })
    }
}

use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

use crate::spec::VolumeIdentifier;

use super::address_appearance_index::Network;

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Deserialize)]
pub enum DataKind {
    AddressAppearanceIndex(Network),
    Sourcify,
    FourByte,
}

impl Default for DataKind {
    fn default() -> Self {
        DataKind::AddressAppearanceIndex(Network::default())
    }
}

pub struct PathPair {
    pub source: PathBuf,
    pub destination: PathBuf,
}
/// Helper for setting up a config.
pub enum DirNature {
    Sample,
    Default,
    Custom(PathPair),
}

impl DirNature {
    /// Combines the SpecId and DirNature enums to get specific dir paths and settings.
    pub fn to_config(self, data: DataKind) -> ConfigStruct {
        match data {
            DataKind::AddressAppearanceIndex(ref n) => match self {
                DirNature::Sample => {
                    ConfigStruct::new(data, "./adapsamplesource", "./adapsampldest")
                }
                DirNature::Default => {
                    ConfigStruct::new(data, "./adapdefaultsource", "./adadefaultdest")
                }
                DirNature::Custom(x) => ConfigStruct {
                    data,
                    source: x.source,
                    destination: x.destination,
                },
            },
            DataKind::Sourcify => todo!(),
            DataKind::FourByte => todo!(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub struct ConfigStruct {
    data: DataKind,
    source: PathBuf,
    destination: PathBuf,
}

impl ConfigStruct {
    pub fn new(data: DataKind, src: &str, dest: &str) -> Self {
        ConfigStruct {
            data,
            source: PathBuf::from(src),
            destination: PathBuf::from(dest),
        }
    }
    /// Gets the base directory for the source data.
    pub fn source_root_dir(&self) -> Result<PathBuf> {
        todo!()
    }
    /// Gets the base directory for the database.
    pub fn dest_root_dir(&self) -> Result<PathBuf> {
        todo!()
    }
    /// Gets the directory that contains Chapter files.
    pub fn chapters_dir(&self) -> Result<PathBuf> {
        todo!()
    }
    /// Gets the path of the manifest file.
    pub fn manifest_file(&self) -> Result<PathBuf> {
        todo!()
    }
    /// Returns the path for a given Chapter.
    pub fn chapter_path(&self) -> Result<PathBuf> {
        todo!()
    }
    /// Returns the VolumeId for the latest Chapter file present.
    pub fn latest_volume(&self) -> Result<VolumeIdentifier> {
        todo!()
    }
}

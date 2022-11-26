use std::path::PathBuf;

use anyhow::{Result};
use serde::Deserialize;

use crate::{
    spec::VolumeIdentifier,
    specs::types::{SpecId},
};
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
    pub fn to_config(self, spec: SpecId) -> ConfigStruct {
        match spec {
            SpecId::AddressAppearanceIndex => match self {
                DirNature::Sample => {
                    ConfigStruct::new(spec, "./adapsamplesource", "./adapsampldest")
                }
                DirNature::Default => {
                    ConfigStruct::new(spec, "./adapdefaultsource", "./adadefaultdest")
                }
                DirNature::Custom(x) => ConfigStruct {
                    spec,
                    source: x.source,
                    destination: x.destination,
                },
            },
            SpecId::Sourcify => todo!(),
            SpecId::FourByte => todo!(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub struct ConfigStruct {
    spec: SpecId,
    source: PathBuf,
    destination: PathBuf,
}

impl ConfigStruct {
    pub fn new(spec: SpecId, src: &str, dest: &str) -> Self {
        ConfigStruct {
            spec,
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

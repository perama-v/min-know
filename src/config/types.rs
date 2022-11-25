use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

use crate::{
    spec::VolumeIdentifier,
    specs::types::{DataSpec, SpecId},
};

use super::address_appearance_index::AdApConfig;

// E.g., path::unchainedpath
pub trait SourceDataPath {
    /// Gets the base directory for the source data.
    fn root_dir(&self) -> Result<PathBuf>;
}

// E.g., path::addressindexpath
pub trait DestinationDataPath {
    /// Gets the base directory for the database.
    fn root_dir(&self) -> Result<PathBuf>;
    /// Gets the directory that contains Chapter files.
    fn chapters_dir(&self) -> Result<PathBuf>;
    /// Gets the path of the manifest file.
    fn manifest_file(&self) -> Result<PathBuf>;
    /// Returns the path for a given Chapter.
    fn chapter_path(&self) -> Result<PathBuf>;
    /// Returns the VolumeId for the latest Chapter file present.
    fn latest_volume(&self) -> Result<VolumeIdentifier>;
}

pub trait DataName {
    fn name() -> String;
}

/// The starting point for setting up a new database.
pub trait DataConfigMethods {
    type Name: DataName;
    type Source: SourceDataPath;
    type Destination: DestinationDataPath;
}

pub enum DirLocation<T: SourceDataPath, U: DestinationDataPath> {
    Sample,
    Default,
    Custom(T, U),
}

impl<T, U> DirLocation<T, U>
where
    T: SourceDataPath,
    U: DestinationDataPath,
{
    /// Combines the SpecId and DirLocation enums to get specific dir paths and settings.
    pub fn to_config(&self, spec: SpecId) -> Result<ConfigsAvailable>
    {
        let data_config: ConfigsAvailable = match spec {
            SpecId::AddressAppearanceIndex => {
                // Build a new config using Sample/Default/Custom.
                let adap_conf = AdApConfig::new(self)?;
                ConfigsAvailable::AdApInConfig(adap_conf)
            }
            SpecId::Sourcify => todo!(),
            SpecId::FourByte => todo!(),
        };
        Ok(data_config)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Deserialize)]
pub enum ConfigsAvailable {
    AdApInConfig(AdApConfig),
    SourcifyConfig(),
}

impl ConfigsAvailable {
    pub fn source_root(&self) -> Result<PathBuf> {
        match self {
            ConfigsAvailable::AdApInConfig(x) => x.source.root_dir(),
            ConfigsAvailable::SourcifyConfig() => todo!(),
        }
    }
}
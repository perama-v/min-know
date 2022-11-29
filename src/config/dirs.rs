use std::{
    fmt::format,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::Deserialize;

use crate::specs::types::{ChapterIdMethods, DataSpec, VolumeIdMethods};

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

impl DataKind {
    fn interface_id(&self) -> String {
        match self {
            DataKind::AddressAppearanceIndex(network) => {
                format!("address_appearance_index_{}", network.name())
            }
            DataKind::Sourcify => format!("sourcify"),
            DataKind::FourByte => format!("four_byte"),
        }
    }
    /// Returns the directory for the index for the given network.
    ///
    /// This directory will contain the index directory (which contains chapter directories).
    /// Conforms to the `ProjectDirs.data_dir()` schema in the Directories crate.
    fn platform_directory(&self) -> Result<PathBuf> {
        let proj_string = match self {
            DataKind::AddressAppearanceIndex(_) => "address-appearance-index",
            DataKind::Sourcify => "sourcify",
            DataKind::FourByte => "four_byte",
        };
        let proj = ProjectDirs::from("", "", proj_string)
            .ok_or_else(|| anyhow!("Could not access env var (e.g., $HOME) to set up project."))?;
        Ok(proj.data_dir().to_path_buf())
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
    pub fn to_config(self, data: DataKind) -> Result<ConfigStruct> {
        let dir_name = data.interface_id();
        let project = data.platform_directory()?;
        Ok(match data {
            DataKind::AddressAppearanceIndex(ref network) => match self {
                DirNature::Sample => ConfigStruct {
                    data,
                    raw_source: PathBuf::from("TODO source sample path"),
                    data_dir: project.join("samples").join(dir_name),
                },
                DirNature::Default => ConfigStruct {
                    data,
                    raw_source: PathBuf::from("TODO source default path"),
                    data_dir: project.join(dir_name),
                },
                DirNature::Custom(x) => ConfigStruct {
                    data,
                    raw_source: x.source.join(&dir_name),
                    data_dir: x.destination.join(&dir_name),
                },
            },
            DataKind::Sourcify => todo!(),
            DataKind::FourByte => todo!(),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub struct ConfigStruct {
    data: DataKind,
    /// The path to the unformatted raw source data. Used for populating the database.
    raw_source: PathBuf,
    /// The path to the functional database.
    data_dir: PathBuf,
}

impl ConfigStruct {
    /// Gets the path of the manifest file.
    pub fn manifest_file(&self) -> Result<PathBuf> {
        todo!()
    }
    /// Returns the path for the directory that holds all chapters that
    /// match the given ChapterId.
    pub fn similar_chapters_path<T: ChapterIdMethods>(&self, chapter: T) -> Result<PathBuf> {
        let mut p = self.data_dir.to_path_buf();
        p.push(chapter.dir_name());
        Ok(p)
    }
    /// Returns the VolumeId for the latest Chapter file present.
    pub fn latest_volume<T: VolumeIdMethods>(&self) -> Result<T> {
        todo!()
    }
}

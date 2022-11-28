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
                    source: PathBuf::from("TODO source sample path"),
                    destination: project.join("samples").join(dir_name),
                },
                DirNature::Default => ConfigStruct {
                    data,
                    source: PathBuf::from("TODO source default path"),
                    destination: project.join(dir_name),
                },
                DirNature::Custom(x) => ConfigStruct {
                    data,
                    source: x.source.join(&dir_name),
                    destination: x.destination.join(&dir_name),
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
    /// The path to the unformatted raw source data. Used for populating
    /// the database.
    source: PathBuf,
    /// The path to the functional database.
    destination: PathBuf,
}

impl ConfigStruct {
    /// Gets the base directory for the source data.
    pub fn source_root_dir(&self) -> Result<PathBuf> {
        Ok(self.source.clone())
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
    pub fn chapter_path<T: ChapterIdMethods>(&self, chapter: T) -> Result<PathBuf> {
        let c = chapter.dir_name();
        let mut p = self.source_root_dir()?;
        p.push(c);
        Ok(p)
    }
    /// Returns the VolumeId for the latest Chapter file present.
    pub fn latest_volume<T: VolumeIdMethods>(&self) -> Result<T> {
        todo!()
    }
}

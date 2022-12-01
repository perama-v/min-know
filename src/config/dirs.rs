use std::{
    fmt::format,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::Deserialize;

use crate::specs::traits::{ChapterIdMethods, DataSpec, VolumeIdMethods};

use super::address_appearance_index::Network;

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Deserialize)]
pub enum DataKind {
    AddressAppearanceIndex(Network),
    Sourcify,
    FourByte
}

impl Default for DataKind {
    fn default() -> Self {
        DataKind::AddressAppearanceIndex(Network::default())
    }
}

impl DataKind {
    fn as_string(&self) -> &str {
        match self {
            DataKind::AddressAppearanceIndex(_) => "address_appearance_index",
            DataKind::Sourcify => "sourcify",
            DataKind::FourByte => "four_byte"
        }
    }
    /// The interface ID is the database kind in string form by default.
    /// Some databases may add additional parameters.
    fn interface_id(&self) -> &str {
        let db_name = self.as_string();
        match self {
            DataKind::AddressAppearanceIndex(network) => {
                format!("{}_{}", db_name, network.name()).as_str()
            }
            _ => db_name,
        }
    }
    /// Returns the directory for the index for the given network.
    ///
    /// This directory will contain the index directory (which contains chapter directories).
    /// Conforms to the `ProjectDirs.data_dir()` schema in the Directories crate.
    fn platform_directory(&self) -> Result<PathBuf> {
        let proj_string = self.as_string();
        let proj = format!("todd_{}", proj_string);
        let proj = ProjectDirs::from("", "", &proj)
            .ok_or_else(|| anyhow!("Could not access env var (e.g., $HOME) to set up project."))?;
        Ok(proj.data_dir().to_path_buf())
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub struct PathPair {
    /// Path for unprocessed data.
    pub raw_source: PathBuf,
    /// Path for processed, formatted, data.
    pub processed_data_dir: PathBuf,
}
/// Helper for setting up a config.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub enum DirNature {
    #[default]
    Sample,
    Default,
    Custom(PathPair),
}

impl DirNature {
    /// Combines the SpecId and DirNature enums to get specific dir paths and settings.
    pub fn to_config(self, data_kind: DataKind) -> Result<ConfigStruct> {
        let dir_name = data_kind.interface_id();
        let project = data_kind.platform_directory()?;
        Ok(match data_kind {
            DataKind::AddressAppearanceIndex(ref network) => match self {
                DirNature::Sample => ConfigStruct {
                    dir_nature: self,
                    data_kind,
                    raw_source: PathBuf::from("TODO source sample path"),
                    data_dir: project.join("samples").join(dir_name),
                },
                DirNature::Default => ConfigStruct {
                    dir_nature: self,
                    data_kind,
                    raw_source: PathBuf::from("TODO source default path"),
                    data_dir: project.join(dir_name),
                },
                DirNature::Custom(ref x) => {
                    let raw_source = x.raw_source.join(&dir_name);
                    let data_dir = x.processed_data_dir.join(&dir_name);
                    ConfigStruct {
                        dir_nature: self,
                        data_kind,
                        raw_source,
                        data_dir,
                    }
                }
            },
            DataKind::Sourcify => todo!(),
            DataKind::FourByte => todo!(),
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub struct ConfigStruct {
    /// Which directory type is being configured. E.g., Real vs sample data.
    pub dir_nature: DirNature,
    /// Which database is being configured.
    pub data_kind: DataKind,
    /// The path to the unformatted raw source data. Used for populating the database.
    pub raw_source: PathBuf,
    /// The path to the functional database.
    pub data_dir: PathBuf,
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

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use super::{address_appearance_index::Network, dirs::ConfigStruct};

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Deserialize, Serialize)]
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
    pub fn as_string(&self) -> &str {
        match self {
            DataKind::AddressAppearanceIndex(_) => "address_appearance_index",
            DataKind::Sourcify => "sourcify",
            DataKind::FourByte => "four_byte",
        }
    }
    /// The interface ID is the database kind in string form by default.
    /// Some databases may add additional parameters.
    pub fn interface_id(&self) -> String {
        let db_name = self.as_string();
        match self {
            DataKind::AddressAppearanceIndex(network) => {
                format!("{}_{}", db_name, network.name())
            }
            _ => db_name.to_string(),
        }
    }
    pub fn raw_source_dir_name(&self) -> String {
        format!("raw_source_{}", self.interface_id())
    }
    /// Returns any parameter within DataKind as a string.
    ///
    /// E.g., AddressAppearanceIndex("mainnet") returns "mainnet".
    pub fn params_as_string(&self) -> Option<&str> {
        match self {
            DataKind::AddressAppearanceIndex(network) => Some(network.name()),
            DataKind::Sourcify => None,
            DataKind::FourByte => None,
        }
    }
    /// Returns the directory for the index for the given network.
    ///
    /// This directory will contain the index directory (which contains chapter directories).
    /// Conforms to the `ProjectDirs.data_dir()` schema in the Directories crate.
    pub fn platform_directory(&self) -> Result<PathBuf> {
        let proj_string = self.as_string();
        let proj = format!("todd_{}", proj_string);
        let proj = ProjectDirs::from("", "", &proj)
            .ok_or_else(|| anyhow!("Could not access env var (e.g., $HOME) to set up project."))?;
        Ok(proj.data_dir().to_path_buf())
    }
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize, Serialize)]
pub struct PathPair {
    /// Path for unprocessed data.
    pub raw_source: PathBuf,
    /// Path for processed, formatted, data.
    pub processed_data_dir: PathBuf,
}
/// Helper for setting up a config.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize, Serialize)]
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
        let raw_dir_name = data_kind.raw_source_dir_name();
        let project = data_kind.platform_directory()?;
        Ok(match data_kind {
            DataKind::AddressAppearanceIndex(ref _network) => match self {
                DirNature::Sample => ConfigStruct {
                    dir_nature: self,
                    base_dir_nature_dependent: project.join("samples"),
                    data_kind,
                    raw_source: project.join("samples").join(raw_dir_name),
                    data_dir: project.join("samples").join(dir_name),
                },
                DirNature::Default => ConfigStruct {
                    dir_nature: self,
                    base_dir_nature_dependent: project.clone(),
                    data_kind,
                    raw_source: project.join(raw_dir_name),
                    data_dir: project.join(dir_name),
                },
                DirNature::Custom(ref x) => {
                    let raw_source = x.raw_source.join(&dir_name);
                    let base_dir_nature_dependent = x.processed_data_dir.clone();
                    let data_dir = x.processed_data_dir.join(&dir_name);
                    ConfigStruct {
                        dir_nature: self.clone(),
                        base_dir_nature_dependent,
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

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
    NameTags,
}

/// Helper for setting up a config.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize, Serialize)]
pub enum DirNature {
    #[default]
    Sample,
    Default,
    Custom(PathPair),
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize, Serialize)]
pub struct PathPair {
    /// Path for unprocessed data.
    pub raw_source: Option<PathBuf>,
    /// Path for processed, formatted, data.
    pub processed_data_dir: Option<PathBuf>,
}

impl DataKind {
    pub(crate) fn as_string(&self) -> &str {
        match self {
            DataKind::AddressAppearanceIndex(_) => "address_appearance_index",
            DataKind::Sourcify => "sourcify",
            DataKind::FourByte => "four_byte",
            DataKind::NameTags => "nametags",
        }
    }
    /// Returns the data kind as a stirng starting with "todd_".
    pub(crate) fn as_todd_string(&self) -> String {
        format!("todd_{}", self.as_string())
    }
    /// The interface ID is the database kind in string form by default.
    /// Some databases may add additional parameters.
    pub(crate) fn interface_id(&self) -> String {
        match self.params_as_string() {
            Some(p) => format!("{}_{}", self.as_string(), p),
            None => self.as_string().to_string(),
        }
    }
    pub(crate) fn raw_source_dir_name(&self) -> String {
        format!("raw_source_{}", self.interface_id())
    }
    /// Returns the inner parameter within DataKind (if present) as a string.
    ///
    /// E.g., AddressAppearanceIndex("mainnet") returns "mainnet".
    pub(crate) fn params_as_string(&self) -> Option<&str> {
        match self {
            DataKind::AddressAppearanceIndex(network) => Some(network.name()),
            _ => None,
        }
    }
    /// Returns the directory for the index for the given network.
    ///
    /// This directory will contain the index directory (which contains chapter directories).
    /// Conforms to the `ProjectDirs.data_dir()` schema in the Directories crate.
    pub(crate) fn platform_directory(&self) -> Result<PathBuf> {
        let proj = ProjectDirs::from("", "", &self.as_todd_string())
            .ok_or_else(|| anyhow!("Could not access env var (e.g., $HOME) to set up project."))?;
        Ok(proj.data_dir().to_path_buf())
    }
}

impl DirNature {
    /// Creates a config, according to the database kind.
    ///
    /// Combines the DataKind and DirNature enums to get specific dir paths and settings.
    pub(crate) fn into_config(self, data_kind: DataKind) -> Result<ConfigStruct> {
        let config = match self {
            DirNature::Sample => self.sample_config(data_kind)?,
            DirNature::Default => self.default_config(data_kind)?,
            DirNature::Custom(ref paths) => self.custom_config(data_kind, paths)?,
        };
        Ok(config)
    }
    /// Used for common pattern of default config setup.
    fn default_config(self, data_kind: DataKind) -> Result<ConfigStruct> {
        let project = data_kind.platform_directory()?;
        Ok(ConfigStruct {
            dir_nature: self,
            base_dir_nature_dependent: project.clone(),
            raw_source: project.join(data_kind.raw_source_dir_name()),
            data_dir: project.join(data_kind.interface_id()),
            data_kind,
        })
    }
    /// Used for common pattern of sample config setup.
    fn sample_config(self, data_kind: DataKind) -> Result<ConfigStruct> {
        let project = data_kind.platform_directory()?;
        Ok(ConfigStruct {
            dir_nature: self,
            base_dir_nature_dependent: project.join("samples"),
            raw_source: project
                .join("samples")
                .join(data_kind.raw_source_dir_name()),
            data_dir: project.join("samples").join(data_kind.interface_id()),
            data_kind,
        })
    }
    /// Used for common pattern of custom config setup.
    ///
    /// Use may pass both, one or none for custom paths in PathPair.
    fn custom_config(&self, data_kind: DataKind, paths: &PathPair) -> Result<ConfigStruct> {
        let project = data_kind.platform_directory()?;
        let base_dir_nature_dependent = match paths.processed_data_dir.clone() {
            Some(p) => p,
            None => project.clone(),
        };
        let raw_source = match paths.raw_source.clone() {
            Some(p) => p,
            None => project.join(data_kind.raw_source_dir_name()),
        };
        let data_dir = base_dir_nature_dependent.join(data_kind.interface_id());
        Ok(ConfigStruct {
            dir_nature: self.clone(),
            base_dir_nature_dependent,
            data_kind,
            raw_source,
            data_dir,
        })
    }
}

#[test]
fn config_default_paths_correctly_formed() {
    let data_kind = DataKind::AddressAppearanceIndex(Network::default());
    let dir_nature = DirNature::Default;
    let config = dir_nature.into_config(data_kind).unwrap();
    let raw = "todd_address_appearance_index/raw_source_address_appearance_index_mainnet";
    assert!(config.raw_source.to_str().unwrap().ends_with(raw));
    let data = "todd_address_appearance_index/address_appearance_index_mainnet";
    assert!(config.data_dir.to_str().unwrap().ends_with(data));
}

#[test]
fn config_sample_paths_correctly_formed() {
    let data_kind = DataKind::AddressAppearanceIndex(Network::default());
    let dir_nature = DirNature::Sample;
    let config = dir_nature.into_config(data_kind).unwrap();
    let raw = "todd_address_appearance_index/samples/raw_source_address_appearance_index_mainnet";
    assert!(config.raw_source.to_str().unwrap().ends_with(raw));
    let data = "todd_address_appearance_index/samples/address_appearance_index_mainnet";
    assert!(config.data_dir.to_str().unwrap().ends_with(data));
}

#[test]
fn config_sample_paths_correct_for_nametags() {
    let config = DirNature::Sample.into_config(DataKind::NameTags).unwrap();
    let raw = "todd_nametags/samples/raw_source_nametags";
    assert!(config.raw_source.to_str().unwrap().ends_with(raw));
    let data = "todd_nametags/samples/nametags";
    assert!(config.data_dir.to_str().unwrap().ends_with(data));
}

#[test]
fn config_custom_paths_correct_for_nametags() {
    let src = "source_dir/test_source_subdir";
    let dst = "dest_dir/test_dest_subdir";
    let paths = PathPair {
        raw_source: Some(PathBuf::from(src)),
        processed_data_dir: Some(PathBuf::from(dst)),
    };
    let config = dbg!(DirNature::Custom(paths)
        .into_config(DataKind::NameTags)
        .unwrap());
    let raw = format!("{}", src);
    assert!(config.raw_source.to_str().unwrap().ends_with(&raw));
    let data = format!("{}/nametags", dst);
    assert!(config.data_dir.to_str().unwrap().ends_with(&data));
}

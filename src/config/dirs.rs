use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result, bail};
use directories::ProjectDirs;
use log::warn;
use serde::Deserialize;

use crate::specs::traits::{ChapterIdMethods, DataSpec, VolumeIdMethods};

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
        let raw_dir_name = data_kind.raw_source_dir_name();
        let project = data_kind.platform_directory()?;
        Ok(match data_kind {
            DataKind::AddressAppearanceIndex(ref network) => match self {
                DirNature::Sample => ConfigStruct {
                    dir_nature: self,
                    data_kind,
                    raw_source: project.join("samples").join(raw_dir_name),
                    data_dir: project.join("samples").join(dir_name),
                },
                DirNature::Default => ConfigStruct {
                    dir_nature: self,
                    data_kind,
                    raw_source: project.join(raw_dir_name),
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
    pub fn chapter_dir_path<T, U>(&self, chapter: &T) -> PathBuf
    where
        T: ChapterIdMethods<U>,
        U: DataSpec,
    {
        let mut p = self.data_dir.to_path_buf();
        p.push(chapter.interface_id());
        p
    }
    /// Returns the VolumeId for the latest Chapter file present.
    pub fn latest_volume<T: DataSpec>(&self) -> Result<T::AssociatedVolumeId>
    {
        // Read the first chapter directory (at random)
        let chapter_dirs = fs::read_dir(&self.data_dir).with_context(|| {
            format!("Couldn't read data directory {:?}.", &self.data_dir)
        })?.next();
        let Some(first) = chapter_dirs else {
            bail!("No chapter directories found in {:?}",
            chapter_dirs)};
        let first = first?.path();
        let chapter = T::AssociatedChapterId::from_chapter_directory(&first)?;
        let vols: Vec<(PathBuf, T::AssociatedVolumeId)> =
            self.parse_all_files_for_chapter::<T>(&chapter)?;
        let mut order: u32 = 0;
        let mut latest = T::AssociatedVolumeId::default();
        for (_path, vol) in vols {
            let current_order = vol.is_nth()?;
            if current_order >= order {
                order = current_order;
                latest = vol
            }
        }
        Ok(latest)
    }
    /// For a given chapter returns the filenames and volume_ids it contains.
    pub fn parse_all_files_for_chapter<T: DataSpec>(&self, chapter: &T::AssociatedChapterId) -> Result<Vec<(PathBuf, T::AssociatedVolumeId)>>
    {
        let chapter_name = chapter.interface_id();
        let dir = self.chapter_dir_path(chapter);
        let files = fs::read_dir(&dir)
            .with_context(|| format!("Couldn't read chapter directory {:?}.", &dir))?;

        let mut all_files: Vec<(PathBuf, T::AssociatedVolumeId)> = vec![];
        for chapterfile in files {
            let file = chapterfile?;
            let filename = file.file_name();
            let Some(filename) = filename.to_str() else {bail!("Couldn't read filename {:?}.", file)};
            // volume_XXX_XXX_XXX_chapter_0xXX.ssz
            // Use knowledge of the chapter directory to get the volume id.
            let volume_str = filename.replace(&chapter_name, "").replace("_.ssz", "");

            let vol_id = T::AssociatedVolumeId::from_interface_id(&volume_str)?;
            all_files.push(
                (file.path(), vol_id)
            )
        }
        warn!("Need to handle all filename suffixes");
        Ok(all_files)
    }
}


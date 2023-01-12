use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::specs::traits::{ChapterIdMethods, DataSpec, VolumeIdMethods};

use super::choices::{DataKind, DirNature};

#[derive(Clone, Debug, PartialEq, PartialOrd, Hash, Deserialize, Serialize)]
pub struct ConfigStruct {
    /// Which directory type is being configured. E.g., Real vs sample data.
    pub dir_nature: DirNature,
    /// The directory that contains the manifest and data_dir. Accounts for
    /// whether data is real/sample.
    pub base_dir_nature_dependent: PathBuf,
    /// Which database is being configured.
    pub data_kind: DataKind,
    /// The path to the unformatted raw source data. Used for populating the database.
    pub raw_source: PathBuf,
    /// The path to the functional database.
    pub data_dir: PathBuf,
}

impl ConfigStruct {
    /// Gets the path of the manifest file.
    pub fn manifest_file_path(&self) -> Result<PathBuf> {
        let mut manifest_filename = self.data_kind.interface_id();
        manifest_filename.push_str("_manifest");
        let mut path = self.base_dir_nature_dependent.join(manifest_filename);
        path.set_extension("json");
        Ok(path)
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
    ///
    /// Assumes that all the Chapter directories contain data for the same Volumes.
    ///
    /// If a Chapter directory has had some files deleted, then this method will
    /// not detect that (unless it is the first directory). This situation is better
    /// navigated using the db.check_completeness() method.
    pub fn latest_volume<T: DataSpec>(&self) -> Result<T::AssociatedVolumeId> {
        // Read the first chapter directory (at random)
        let chapter_dirs = fs::read_dir(&self.data_dir)
            .with_context(|| format!("Couldn't read data directory {:?}.", &self.data_dir))?
            .next();
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
    pub fn parse_all_files_for_chapter<T: DataSpec>(
        &self,
        chapter: &T::AssociatedChapterId,
    ) -> Result<Vec<(PathBuf, T::AssociatedVolumeId)>> {
        let chapter_name = chapter.interface_id();
        let dir = self.chapter_dir_path(chapter);
        let files = fs::read_dir(&dir)
            .with_context(|| format!("Couldn't read chapter directory {:?}.", &dir))?;

        let mut all_files: Vec<(PathBuf, T::AssociatedVolumeId)> = vec![];
        for chapterfile in files {
            let file = chapterfile?;
            let filename = file.file_name();
            let Some(filename) = filename.to_str() else {bail!("Couldn't read filename {:?}.", file)};
            let without_chapter = filename.replace(&chapter_name, "");
            let Some((volume_str, _suffix)) = without_chapter.split_once("_.") else {
                bail!("Filename could not be split by '_' and '.': {}", filename)};
            let vol_id = T::AssociatedVolumeId::from_interface_id(volume_str)?;
            all_files.push((file.path(), vol_id))
        }
        Ok(all_files)
    }
    /// Gets the path of the local repository sample data.
    fn local_sample_base_dir(&self) -> PathBuf {
        PathBuf::from("./data/samples").join(self.data_kind.as_todd_string())
    }
    /// Gets the path of the local repository processed sample data.
    pub fn local_sample_data_dir(&self) -> PathBuf {
        self.local_sample_base_dir()
            .join(self.data_kind.as_string())
    }
    /// Gets the path of the local repository raw sample data.
    pub fn local_sample_raw_source(&self) -> PathBuf {
        self.local_sample_base_dir()
            .join(self.data_kind.raw_source_dir_name())
    }
}

#[test]
fn config_local_paths_correct_for_nametags() {
    let config = DirNature::Sample.into_config(DataKind::NameTags).unwrap();
    let raw = "/data/samples/todd_nametags/raw_source_nametags";
    let path = dbg!(config.local_sample_raw_source());
    assert!(path.to_str().unwrap().ends_with(raw));
    let data = "/data/samples/todd_nametags/nametags";
    let path = dbg!(config.local_sample_data_dir());
    assert!(path.to_str().unwrap().ends_with(data));
}

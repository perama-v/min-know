use std::{
    env::current_dir,
    ffi::OsString,
    fs::{self},
    path::PathBuf,
};

use anyhow::{anyhow, Context, Result};

pub trait DirFunctions {
    /// Determines if a directory contains all the filenames provided.
    ///
    /// E.g., Use to check if samples are present in sample directory.
    fn contains_files(&self, files: &Vec<&'static str>) -> Result<bool>;

    /// Copies the source directory files into destination directory.
    ///
    /// source/file1 -> dest/file1
    fn copy_into_recursive(&self, destination: &PathBuf) -> Result<()>;
}
impl DirFunctions for PathBuf {
    fn contains_files(&self, files: &Vec<&'static str>) -> Result<bool> {
        let Ok(contents) = fs::read_dir(self) else {return Ok(false)};
        let present: Vec<String> = contents
            .into_iter()
            .filter_map(|x| x.ok())
            .map(|x| x.file_name())
            .map(|x| x.to_os_string())
            .filter_map(|x| x.into_string().ok())
            .collect();
        for desired in files {
            if !present.contains(&desired.to_string()) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn copy_into_recursive(&self, destination: &PathBuf) -> Result<()> {
        fs::create_dir_all(&destination)?;
        println!(
            "Copying files within directory:\n\t{:?} into directory:\n\t{:?}",
            self, destination
        );
        //todo!("Remove me once satisfied paths are configure correctly.");
        for entry in fs::read_dir(self)? {
            let entry = entry?;
            let entry_type = entry.file_type()?;
            if entry_type.is_dir() {
                entry
                    .path()
                    .copy_into_recursive(&destination.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), destination.join(entry.file_name()))?;
            }
        }
        Ok(())
    }
}

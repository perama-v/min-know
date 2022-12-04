use std::{
    env::current_dir,
    fs::{self},
    path::PathBuf,
};

use anyhow::{anyhow, Result};

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
        println!("Looking for samples in {:?}", self);
        for sample in fs::read_dir(self)? {
            let f = sample?.file_name();
            let Some(filename) = f.to_str() else {
                return
                Err(anyhow!("Unable to read a file in {:?}", self))
            };
            if !files.contains(&filename) {
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

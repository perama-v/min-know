use anyhow::Result;


use std::{
    fmt::Display,
    fs::{self},
    path::PathBuf,
};

pub trait DirFunctions {
    /// Determines if a directory contains all the filenames provided.
    ///
    /// E.g., Use to check if samples are present in sample directory.
    fn contains_files<T: AsRef<str> + Display>(&self, files: &[T]) -> Result<bool>;

    /// Copies the source directory files into destination directory.
    ///
    /// source/file1 -> dest/file1
    fn copy_into_recursive(&self, destination: &PathBuf) -> Result<()>;
}
impl DirFunctions for PathBuf {
    // test<T: AsRef<str>>(inp: &[T]) {
    fn contains_files<T: AsRef<str> + Display>(&self, files: &[T]) -> Result<bool> {
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

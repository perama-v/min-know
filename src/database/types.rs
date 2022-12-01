use anyhow::{anyhow, Context, Result};
use std::{fmt::Debug, fs};

use serde::Deserialize;

use crate::{
    config::dirs::{ConfigStruct, DataKind, DirNature},
    samples::traits::SampleObtainer,
    specs::traits::{ChapterMethods, DataSpec, RecordMethods, RecordValueMethods},
};

/// The definition for the entire new database.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Todd<T: DataSpec> {
    pub chapters: Vec<T::AssociatedChapter>,
    pub config: ConfigStruct,
}

/// Implement generic methods common to all databases.
impl<T: DataSpec> Todd<T> {
    pub fn new(specification: DataKind, directories: DirNature) -> Result<Self> {
        // Use the spec to then get the DataConfig.
        let config = directories.to_config(specification)?;
        Ok(Self {
            chapters: vec![],
            config,
        })
    }
    // Creates new and complete todd.
    pub fn full_transform<V>(&mut self) -> Result<()> {
        let chapts = T::get_all_chapter_ids();
        let vols = T::get_all_volume_ids();
        for chapter in &chapts {
            for vol in &vols {
                let chapter = self.get_one_chapter::<V>(vol, chapter)?;
                self.save_chapter(chapter);
            }
        }
        Ok(())
    }
    pub fn spec_name(&self) -> &str {
        T::DATABASE_INTERFACE_ID
    }
    pub fn chapter_interface_id(&self, chapter: T) -> String {
        T::chapter_interface_id(chapter)
    }
    /// Prepares the mininum distributable Chapter
    pub fn get_one_chapter<V>(
        &self,
        vol: &T::AssociatedVolumeId,
        chapter: &T::AssociatedChapterId,
    ) -> Result<T::AssociatedChapter> {
        let mut vals: Vec<T::AssociatedRecord> = vec![];
        let source_data: Vec<(&str, V)> = self.raw_pairs();
        for (raw_key, raw_val) in source_data {
            let record_key = T::raw_key_as_record_key(raw_key)?;
            if T::record_key_matches_chapter(&record_key, &vol, &chapter) {
                let record_value = T::raw_value_as_record_value(raw_val).get();
                let rec: T::AssociatedRecord = <T::AssociatedRecord>::new(record_key, record_value);
                vals.push(rec)
            }
        }
        let mut chapter = T::new_chapter();
        Ok(chapter)
    }
    pub fn raw_pairs<V>(&self) -> Vec<(&str, V)> {
        // A vector of generic key-value pairs.
        // E.g., (address, appearances) or (address, ABIs)
        todo!()
    }
    pub fn save_chapter(&self, c: T::AssociatedChapter) {}
    /// Obtains the RecordValues that match a particular RecordKey
    ///
    /// Each Chapter contains Records with key-value pairs. This function
    /// aggregates values from all relevant Records (across different Chapters).
    pub fn find(&self, raw_record_key: &str) -> Result<Vec<String>> {
        let target_record_key = T::raw_key_as_record_key(raw_record_key)?;
        let chapter_id = T::record_key_to_chapter_id(&target_record_key)?;
        let chap_dir = self.config.similar_chapters_path(chapter_id)?;
        // Read each file and collect matching Values
        let files = fs::read_dir(&chap_dir)
            .with_context(|| format!("Failed to read dir {:?}", chap_dir))?;
        let mut matching: Vec<String> = vec![];
        for filename in files {
            let path = filename?.path();
            let bytes =
                fs::read(&path).with_context(|| format!("Failed to read files from {:?}", path))?;
            let chapter = <T::AssociatedChapter>::from_file(bytes)?;
            let records = chapter.records();
            for r in records {
                let rec = r.get();
                let key = rec.key();
                if key == &target_record_key {
                    matching.extend(r.values_as_strings())
                }
            }
        }
        Ok(matching)
    }
    /// Obtains the sample data for the database.
    ///
    /// Samples may be in the cross-platform path (Directories crate),
    /// the local folder (if repo is cloned from GH) or may need
    /// to be obtained from a custom source.
    pub fn get_sample_data(&self) -> Result<()> {
        if let DirNature::Sample = self.config.dir_nature {
        } else {
            return Err(anyhow!("try to configure the db with DirNature::Sample"));
        }
        if T::AssociatedSampleObtainer::raw_samples_present(self.config.raw_source) {
            println!(
                "The sample files are already present in {:?}",
                self.config.raw_source
            );
        } else {
            let repo_dir = self.config.raw_source.as_local_repo();
            if T::AssociatedSampleObtainer::raw_samples_present(repo_dir) {
                copy_raw_samples_from_repo(repo_dir)
            } else {
                T::AssociatedSampleObtainer::get_raw_samples()
            }
        }

        if T::AssociatedSampleObtainer::processed_samples_present(self.config.processed_data_dir) {
            println!(
                "The sample files are already present in {:?}",
                self.config.processed_data_dir
            );
        } else {
            let repo_dir = self.config.processed_data_dir.as_local_repo();
            if T::AssociatedSampleObtainer::processed_samples_present(repo_dir) {
                copy_processed_samples_from_repo(repo_dir)
            } else {
                T::AssociatedSampleObtainer::get_processed_samples()
            }
        }
        Ok(())
    }
}

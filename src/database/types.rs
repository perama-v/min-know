use anyhow::{anyhow, Context, Result};
use std::{fmt::Debug, fs, path::PathBuf};

use serde::Deserialize;

use crate::{
    config::dirs::{ConfigStruct, DataKind, DirNature},
    extraction::traits::Extractor,
    samples::traits::SampleObtainer,
    specs::traits::{
        ChapterIdMethods, ChapterMethods, DataSpec, RecordMethods, RecordValueMethods,
    },
};

use super::utils::{self, DirFunctions};

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
    /// Creates new and complete TODD-compliant database from
    /// a specification and corresponding raw data source.
    ///
    /// ## Example
    /// ```ignore
    /// let mut db: Todd<AAISpec> = Todd::new(DataKind::default(), DirNature::Sample)?;
    /// db.full_transform()?;
    /// ```
    /// ## Algorithm
    /// Relies on the existence of an Extractor method that each database must implement.
    /// That method raw source data in the specified directory and produces a Chapter
    /// that matches the specified VolumeId and ChapterId.
    ///
    /// The returned Chapter is then saved.
    /// This is repeated for all possible Chapters and may occur in parallel.
    pub fn full_transform<V>(&mut self) -> Result<()> {
        let chapts = T::get_all_chapter_ids();
        let vols = T::get_all_volume_ids();
        for chapter_id in &chapts {
            for volume_id in &vols {
                // todo!("Extraction by fetching iterators over relevant source db.");
                let chapter: T::AssociatedChapter = T::AssociatedExtractor::chapter_from_raw(
                    chapter_id,
                    volume_id,
                    &self.config.raw_source,
                )?;
                self.save_chapter(chapter);
            }
        }
        Ok(())
    }
    pub fn chapter_interface_id(&self, chapter: T::AssociatedChapter) -> String {
        chapter.chapter_id().interface_id().to_owned()
    }
    /// Prepares the mininum distributable Chapter
    pub fn deprecated_get_one_chapter<V>(
        &self,
        vol: &T::AssociatedVolumeId,
        chapter: &T::AssociatedChapterId,
    ) -> Result<T::AssociatedChapter> {
        let mut vals: Vec<T::AssociatedRecord> = vec![];
        let source_data: Vec<(&str, V)> = self.deprecated_raw_pairs();
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
    pub fn deprecated_raw_pairs<V>(&self) -> Vec<(&str, V)> {
        // A vector of generic key-value pairs.
        // E.g., (address, appearances) or (address, ABIs)
        todo!()
    }
    fn save_chapter(&self, chapter: T::AssociatedChapter) {
        todo!("Save chapter to file.")
    }
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
    /// This includes processed (TODD-compliant) samples and raw samples
    /// that can be used to create processed samples.
    ///
    /// Samples may be in the cross-platform path (Directories crate),
    /// the local folder (if repo is cloned from GH) or may need
    /// to be obtained from a custom source. This method tries each in that
    /// order.
    ///
    /// ## Example
    /// ```
    /// # use anyhow::Result;
    /// # use min_know::{
    /// #    config::dirs::{DataKind, DirNature},
    /// #    database::types::Todd,
    /// #    specs::address_appearance_index::AAISpec,
    /// # };
    /// let db: Todd<AAISpec> = Todd::new(DataKind::default(), DirNature::Sample)?;
    /// db.get_sample_data()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get_sample_data(&mut self) -> Result<()> {
        if let DirNature::Sample = self.config.dir_nature {
        } else {
            return Err(anyhow!("try to configure the db with DirNature::Sample"));
        }
        let example_dir_raw =
            PathBuf::from("./data/samples").join(self.config.data_kind.raw_source_dir_name());
        let example_dir_processed =
            PathBuf::from("./data/samples").join(self.config.data_kind.interface_id());

        let raw_sample_filenames = T::AssociatedSampleObtainer::raw_sample_filenames();
        let processed_sample_filenames = T::AssociatedSampleObtainer::processed_sample_filenames();
        // Raw samples
        if !self
            .config
            .raw_source
            .contains_files(&raw_sample_filenames)?
        {
            if example_dir_raw.contains_files(&raw_sample_filenames)? {
                example_dir_raw.copy_into_recursive(&self.config.raw_source)?;
            } else {
                T::AssociatedSampleObtainer::get_raw_samples(&self.config.raw_source)?
            }
        }

        // Processed samples
        match processed_sample_filenames {
            Some(filenames) => {
                if self.config.data_dir.contains_files(&filenames)? {
                    return Ok(());
                } else {
                    if example_dir_processed.contains_files(&filenames)? {
                        example_dir_processed.copy_into_recursive(&self.config.data_dir)?;
                        return Ok(());
                    }
                }
            }
            None => {}
        };
        // Create the samples by processing the raw samples.
        self.full_transform::<T>()?;
        Ok(())
    }
}

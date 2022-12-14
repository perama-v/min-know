use anyhow::{anyhow, Context, Result};
use std::{
    fmt::Debug,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use log::{debug, error, info, warn};
use rayon::prelude::*;
use serde::Deserialize;

use crate::{
    config::dirs::{ConfigStruct, DataKind, DirNature},
    extraction::traits::Extractor,
    samples::traits::SampleObtainer,
    specs::traits::{
        ChapterIdMethods, ChapterMethods, DataSpec, RecordMethods, RecordValueMethods,
        VolumeIdMethods,
    },
};

use super::utils::DirFunctions;

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
    ///
    pub fn full_transform<V>(&mut self) -> Result<()> {
        let chapter_ids = &T::get_all_chapter_ids()?;
        let volume_ids = &T::get_all_volume_ids(&self.config.raw_source)?;
        info!(
            "There are {} volumes, each with {} chapters.",
            volume_ids.len(),
            chapter_ids.len()
        );
        let total_chapters = chapter_ids.len() * volume_ids.len();
        let count = Arc::new(Mutex::new(0_u32));

        volume_ids.par_iter().for_each(|volume_id| {
            chapter_ids.par_iter().for_each(|chapter_id| {
                self.create_chapter(&volume_id, &chapter_id);
                {
                    let mut c = count.lock().unwrap();
                    *c += 1;
                    if *c % 100 == 0 {
                        info!(
                            "Finished checking/creating chapter {} of {}",
                            c, total_chapters
                        )
                    }
                }
            })
        });
        Ok(())
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
        let chapter = todo!("(deprecated) previously: T::new_chapter()");
        Ok(chapter)
    }
    pub fn deprecated_raw_pairs<V>(&self) -> Vec<(&str, V)> {
        // A vector of generic key-value pairs.
        // E.g., (address, appearances) or (address, ABIs)
        todo!()
    }
    /// Creates then saves a single chapter.
    ///
    /// ## Errors
    /// All errors encountered during child function execution are handled
    /// by logging here (no errors are returned). This is to enable the
    /// function to be called concurrently.
    fn create_chapter(
        &self,
        volume_id: &T::AssociatedVolumeId,
        chapter_id: &T::AssociatedChapterId,
    ) {
        let chapter = T::AssociatedExtractor::chapter_from_raw(
            &chapter_id,
            volume_id,
            &self.config.raw_source,
        );
        let v_id = volume_id.interface_id();
        let c_id = chapter_id.interface_id();
        match chapter {
            Err(e) => error!(
                "Error processing chapter (vol_id: {:?}, chap_id: {:?}): {}",
                v_id, c_id, e
            ),
            Ok(chap_opt) => match chap_opt {
                None => { /* No raw data for this volume_id/chapter_id combo (skip). */ }
                Some(chap) => match self.save_chapter(chap) {
                    Ok(_) => {}
                    Err(e) => error!(
                        "Error processing chapter (vol_id: {:?}, chap_id: {:?}): {}",
                        v_id, c_id, e
                    ),
                },
            },
        };
    }
    fn save_chapter(&self, chapter: T::AssociatedChapter) -> Result<()> {
        let chapter_dir_path = &self
            .config
            .data_dir
            .join(&chapter.chapter_id().interface_id());
        fs::create_dir_all(chapter_dir_path)?;
        let encoded = chapter.as_serialized_bytes();
        let filename = chapter.filename();
        debug!(
            "Saving chapter: {}, with {} records ({} bytes).",
            &filename,
            chapter.records().len(),
            encoded.len()
        );
        let filepath = chapter_dir_path.join(&filename);
        fs::write(&filepath, encoded).context(anyhow!("Unable to write file {:?}", &filepath))?;
        Ok(())
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
                    matching.extend(r.clone().values_as_strings())
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
    /// The processed samples may need to be created from the raw samples, which
    /// can be slow.
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
        self.handle_raw_samples()?;
        self.handle_database_samples()?;
        Ok(())
    }
    /// Ensures that the unprocessed samples are either present or obtained.
    fn handle_raw_samples(&self) -> Result<()> {
        let raw_source_dir = &self.config.raw_source;
        let local_example_dir_raw =
            PathBuf::from("./data/samples").join(self.config.data_kind.raw_source_dir_name());
        let raw_sample_filenames = T::AssociatedSampleObtainer::raw_sample_filenames();

        if raw_source_dir.contains_files(&raw_sample_filenames)? {
            info!("Checking raw sample files: already present.");
            return Ok(());
        }

        if local_example_dir_raw.contains_files(&raw_sample_filenames)? {
            info!("Raw sample files found in local repository: moving to samples directory.");
            local_example_dir_raw.copy_into_recursive(&raw_source_dir)?;
        } else {
            info!("Raw samples not found: downloading.");
            T::AssociatedSampleObtainer::get_raw_samples(&raw_source_dir)?
        }
        Ok(())
    }
    /// Ensures that the processed samples are either present or obtained.
    ///
    /// First looks in the expected location, then looks in the local
    /// directory (and copies if present), then attempts to processes from raw
    /// data.
    fn handle_database_samples(&mut self) -> Result<()> {
        let example_dir_processed =
            PathBuf::from("./data/samples").join(self.config.data_kind.interface_id());

        let volume_interface_ids = T::AssociatedSampleObtainer::sample_volumes();

        let Some(volume_interface_ids) = volume_interface_ids else {
            info!("No sample filenames provided: creating samples from raw data.");
            self.full_transform::<T>()?;
            return Ok(())
        };
        let volume_ids = volume_interface_ids
            .iter()
            .map(|x| T::AssociatedVolumeId::from_interface_id(x))
            .collect::<Result<Vec<T::AssociatedVolumeId>>>();

        let Ok(volume_ids) = volume_ids else {
                warn!("Couldn't derive VolumeId from provided interface id: skipping check for existing samples.");
                self.full_transform::<T>()?;
                return Ok(())
            };
        // Chapter directories as (directory_name, filenames)
        let mut dirnames_and_files: Vec<(String, Vec<String>)> = vec![];
        for i in 0..T::NUM_CHAPTERS {
            let Ok(chapter_id) = T::AssociatedChapterId::nth_id(i as u32) else {
                warn!("Couldn't derive nth ChapterId: skipping check for existing samples.");
                self.full_transform::<T>()?;
                return Ok(())
            };
            let mut filenames: Vec<String> = vec![];
            for volume_id in &volume_ids {
                let filename = T::AssociatedChapter::new_empty(volume_id, &chapter_id).filename();
                filenames.push(filename);
            }
            dirnames_and_files.push((chapter_id.interface_id(), filenames));
        }
        // Check expected location.
        let mut data_dir_complete = true;
        for (dirname, filenames) in &dirnames_and_files {
            let chap_dir = self.config.data_dir.join(&dirname);
            // Detect if any of the sample files are missing.
            if !chap_dir.contains_files(&filenames)? {
                data_dir_complete = false
            }
        }
        if data_dir_complete {
            info!("Sample directory already contains all database samples.");
            return Ok(());
        }
        // Check local directory.
        let mut local_data_dir_complete = true;
        for (dirname, filenames) in &dirnames_and_files {
            let chap_dir = example_dir_processed.join(&dirname);
            // Detect if any of the sample files are missing.
            if !chap_dir.contains_files(&filenames)? {
                local_data_dir_complete = false
            }
        }
        if local_data_dir_complete {
            info!("Local directory has sample files: copying to samples directory.");
            for (dirname, _filenames) in &dirnames_and_files {
                let src_chap_dir = example_dir_processed.join(&dirname);
                let dest_chap_dir = self.config.data_dir.join(&dirname);
                src_chap_dir.copy_into_recursive(&dest_chap_dir)?;
            }
            return Ok(());
        } else {
            info!("Local directory does not contain sample files: creating from raw data.");
            self.full_transform::<T>()?;
        }
        Ok(())
    }
}

use anyhow::{anyhow, bail, Context, Result};
use reqwest::Url;
use std::{
    fmt::Debug,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;

use log::{debug, error, info, warn};
use rayon::prelude::*;
use serde::Deserialize;

use crate::{
    config::dirs::{ConfigStruct, DataKind, DirNature},
    database::utils::log_count,
    extraction::traits::Extractor,
    ipfs::cid_v0_string_from_bytes,
    samples::{
        traits::SampleObtainer,
        utils::{download_files, DownloadTask},
    },
    specs::traits::{
        ChapterIdMethods, ChapterMethods, DataSpec, ManifestMethods, RecordMethods,
        RecordValueMethods, VolumeIdMethods,
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
    /// Initialise the database library with the given configuration.
    pub fn init(specification: DataKind, directories: DirNature) -> Result<Self> {
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
    /// let mut db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample)?;
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
    pub fn full_transform(&self) -> Result<()> {
        let volume_ids = &T::get_all_volume_ids(&self.config.raw_source)?;
        let chapter_ids = &T::get_all_chapter_ids()?;
        self.create_listed_chapters(volume_ids, chapter_ids)?;
        info!("Finished creating database.");
        self.generate_manifest()?;
        Ok(())
    }
    /// Extends the database by transforming unincorporated raw data.
    ///
    /// ## Algorithm
    /// - First new VolumeId: Find the latest volume that exists, increment, get VolumeId.
    /// - Latest new VolumeId:
    ///     - Get all VolumeIds possible based on raw data (use extractor methods)
    ///     - Get the latest VolumeId present in processed data.
    ///     - Remove existing from the list of possible.
    /// - Get list of vol_ids in range: first..=last.
    /// - For vol_ids/chapter_ids combinations, self.create_chapter
    /// - Generate manifest unless changes were None.
    ///
    /// ## Database specific concepts
    ///
    /// For each database, the latest volume can be found from raw data properties:
    /// - AAI: Block number of the latest chunk is used.
    /// - Nametag: Index of the last file in the append-only raw database.
    ///     - Edits are appended not added as a new entry, not included in the exsisting file.
    ///     - All entries have an index. The index of the latest entry is used.
    /// - Contract source code: The index of the latest entry is used.
    /// - 4 byte signature: The index of the latest entry is used.
    pub fn extend(&self) -> Result<()> {
        let all_possible_volume_ids = T::get_all_volume_ids(&self.config.raw_source)?;

        let latest_existing_vol = self.config.latest_volume::<T>()?;
        let index_of_existing = latest_existing_vol.is_nth()? as usize;

        let mut new_volume_ids: Vec<T::AssociatedVolumeId> = vec![];
        for (index, vol) in all_possible_volume_ids.into_iter().enumerate() {
            if index > index_of_existing {
                new_volume_ids.push(vol);
            }
        }
        let chapter_ids = &T::get_all_chapter_ids()?;
        self.create_listed_chapters(&new_volume_ids, chapter_ids)?;
        info!("Finished extending database.");
        self.generate_manifest()?;
        Ok(())
    }
    /// Creates every possible Chapter using the VolumeIds/ChapterIds provided.
    ///
    /// Every combination of is created.
    ///
    /// Used by self.full_transform() and self.extend().
    fn create_listed_chapters(
        &self,
        volume_ids: &Vec<T::AssociatedVolumeId>,
        chapter_ids: &Vec<T::AssociatedChapterId>,
    ) -> Result<()> {
        let total_chapters = (chapter_ids.len() * volume_ids.len()) as u32;
        info!(
            "{} VolumeIds, each with {} ChapterIds is {} total Chapters.",
            volume_ids.len(),
            chapter_ids.len(),
            total_chapters
        );
        let count = Arc::new(Mutex::new(0_u32));

        volume_ids.par_iter().for_each(|volume_id| {
            chapter_ids.par_iter().for_each(|chapter_id| {
                self.create_chapter(&volume_id, &chapter_id);
                log_count(
                    count.clone(),
                    total_chapters,
                    "Finished checking/creating chapter",
                    100,
                );
            })
        });
        Ok(())
    }
    /// Creates a new manifest file.
    ///
    /// This will override an existing manifest file. The file
    /// in the directory alongside the data and raw data directories.
    ///
    /// ## Algorithm
    /// 1. Goes through each Chapter file in the data directory.
    /// 2. The IPFS CID (v0) is computed from the file bytes as-is (encoded).
    /// 3. Additional database metadata is recorded.
    /// 4. File is saved as a {database_interface_id}_manifest.json.
    pub fn generate_manifest(&self) -> Result<()> {
        info!("Generating manifest.");
        let mut manifest = T::AssociatedManifest::default();
        let mut cids: Vec<(String, T::AssociatedVolumeId, T::AssociatedChapterId)> = vec![];
        // Go through all the files in config.data_dir
        let chapter_dirs = fs::read_dir(&self.config.data_dir).with_context(|| {
            format!("Couldn't read data directory {:?}.", &self.config.data_dir)
        })?;
        for chapter_dir in chapter_dirs {
            // Obtain ChapterId from directory name.
            let dir = chapter_dir?.path();
            let chap_id = T::AssociatedChapterId::from_chapter_directory(&dir)?;
            // Obtain VolumeIds using ChapterId
            let chapter_files: Vec<(PathBuf, T::AssociatedVolumeId)> =
                self.config.parse_all_files_for_chapter::<T>(&chap_id)?;
            for (chapter_path, volume_id) in chapter_files {
                let bytes = fs::read(chapter_path)?;
                let cid = cid_v0_string_from_bytes(&bytes)?;
                cids.push((cid, volume_id, chap_id.clone()))
            }
        }
        let latest_volume: T::AssociatedVolumeId = self.config.latest_volume::<T>()?;
        // For each file get filename (--> volume_id and chapter_id) and bytes
        // CID from bytes
        manifest.set_spec_version(T::spec_version());
        manifest.set_schemas(T::spec_schemas_resource());
        manifest.set_database_interface_id(self.config.data_kind.interface_id());
        manifest.set_latest_volume_identifier(latest_volume.interface_id());
        manifest.set_cids(&cids);

        let manifest_path = self.config.manifest_file_path()?;
        let json_manifest = serde_json::to_string_pretty(&manifest)?;

        fs::write(&manifest_path, json_manifest)
            .with_context(|| format!("Failed to write file: {:?}", &manifest_path))?;
        debug!("Manifest saved.");
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
                let rec: T::AssociatedRecord = T::AssociatedRecord::new(record_key, record_value);
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
        let chapter_result = T::AssociatedExtractor::chapter_from_raw(
            &chapter_id,
            volume_id,
            &self.config.raw_source,
        );
        let current_chapter = format!(
            "chapter (vol_id: {:?}, chap_id: {:?})",
            volume_id.interface_id(),
            chapter_id.interface_id()
        );

        let chapter_option = match chapter_result {
            Ok(c) => c,
            Err(e) => {
                error!("Error processing {}: {}", current_chapter, e);
                return;
            }
        };

        let Some(chapter) = chapter_option else {
            /* No raw data for this volume_id/chapter_id combo (skip). */
            return
        };

        match self.save_chapter(chapter) {
            Ok(_) => {}
            Err(e) => error!("Error processing {}: {}", current_chapter, e),
        }
    }
    fn save_chapter(&self, chapter: T::AssociatedChapter) -> Result<()> {
        let chapter_dir_path = &self.config.chapter_dir_path(chapter.chapter_id());
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
        let chap_dir = self.config.chapter_dir_path(&chapter_id);
        // Read each file and collect matching Values
        let files = fs::read_dir(&chap_dir)
            .with_context(|| format!("Failed to read dir {:?}", chap_dir))?;
        let mut matching: Vec<String> = vec![];
        for filename in files {
            let path = filename?.path();
            debug!("Reading file: {:?}", path);
            let bytes =
                fs::read(&path).with_context(|| format!("Failed to read file from {:?}", path))?;
            let chapter = <T::AssociatedChapter>::from_file(bytes)
                .with_context(|| format!("Failed to read/decode file: {:?}", path))?;
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
    /// Acquires the parts of the database that a user would be interested in.
    ///
    /// The user provides the database keys important to them. This is used
    /// locally to determine which Chapters are relevant. Those Chapters
    /// are then downloaded using the CIDs present in the local manifest file.
    ///
    /// ## Algorithm
    ///
    /// 1. Convert the raw keys into ChapterIds.
    /// 2. Go through all the Chapter CIDs in the manifest.
    /// 3. Keep Chapter CIDs that match the ChapterIds from the raw keys.
    /// 4. Use the CIDs to download the Chapters and save locally.
    pub fn obtain_relevant_data(&self, keys: &[&str], gateway: &str) -> Result<()> {
        warn!("TODO: Manifest should be downloaded not sourced locally.");

        let mut relevant_chapter_ids: Vec<T::AssociatedChapterId> = vec![];
        for k in keys {
            let record_key = T::raw_key_as_record_key(&k)?;
            let chapter_id = T::record_key_to_chapter_id(&record_key)?;
            relevant_chapter_ids.push(chapter_id);
        }

        let path = self.config.manifest_file_path()?;
        let str = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read manifest: {:?}", &path))?;
        let manifest: T::AssociatedManifest = serde_json::from_str(&str)?;

        let mut tasks: Vec<DownloadTask> = vec![];
        for (cid, vol_id, chap_id) in manifest.cids()? {
            if relevant_chapter_ids.contains(&chap_id) {
                let url = Url::parse(gateway)?.join(cid)?;
                let dest_dir = self.config.chapter_dir_path(&chap_id);
                let filename = T::AssociatedChapter::new_empty(&vol_id, &chap_id).filename();
                tasks.push(DownloadTask {
                    url,
                    dest_dir,
                    filename,
                })
            }
        }
        let rt = Runtime::new()?;
        rt.block_on(download_files(tasks))?;
        info!("TODO: Downloaded data can now be pinned on IPFS to support the network.");
        Ok(())
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
    /// let db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample)?;
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
            self.full_transform()?;
            return Ok(())
        };
        let volume_ids = volume_interface_ids
            .iter()
            .map(|x| T::AssociatedVolumeId::from_interface_id(x))
            .collect::<Result<Vec<T::AssociatedVolumeId>>>();

        let Ok(volume_ids) = volume_ids else {
                warn!("Couldn't derive VolumeId from provided interface id: skipping check for existing samples.");
                self.full_transform()?;
                return Ok(())
            };
        // Chapter directories as (directory_name, filenames)
        let mut dirnames_and_files: Vec<(String, Vec<String>)> = vec![];
        for i in 0..T::NUM_CHAPTERS {
            let Ok(chapter_id) = T::AssociatedChapterId::nth_id(i as u32) else {
                warn!("Couldn't derive nth ChapterId: skipping check for existing samples.");
                self.full_transform()?;
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
            self.full_transform()?;
        }
        Ok(())
    }
}

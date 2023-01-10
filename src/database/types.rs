use anyhow::{anyhow, Context, Result};
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
use serde::{Deserialize, Serialize};

use crate::{
    config::{
        choices::{DataKind, DirNature},
        dirs::ConfigStruct,
    },
    extraction::traits::ExtractorMethods,
    samples::traits::SampleObtainerMethods,
    specs::traits::{
        ChapterIdMethods, ChapterMethods, DataSpec, ManifestMethods, RecordMethods, VolumeIdMethods,
    },
    utils::{
        download::{download_files, DownloadTask},
        ipfs::cid_v0_string_from_bytes,
        system::DirFunctions,
    },
};

/// The definition for the entire new database.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Todd<T: DataSpec> {
    chapters: Vec<T::AssociatedChapter>,
    pub config: ConfigStruct,
}

/// Implement generic methods common to all databases.
impl<T: DataSpec> Todd<T> {
    /// Initialise the database library with the given configuration.
    pub fn init(data_kind: DataKind, directories: DirNature) -> Result<Self> {
        assert!(
            T::spec_matches_input(&data_kind),
            "DataKind does not match Spec type"
        );

        // Use the spec to then get the DataConfig.
        let config = directories.to_config(data_kind)?;
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
        self.create_chapter_combinations(volume_ids, chapter_ids)?;
        info!("Finished creating database.");
        self.generate_manifest()?;
        Ok(())
    }
    /// Extends the database by transforming unincorporated raw data.
    ///
    /// ## Algorithm
    /// - Get the latest VolumeId present in processed data.
    /// - Get all VolumeIds possible based on raw data (use extractor methods)
    /// - Keep only the VolumeIds that are later than the latest existing VolumeId.
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
        self.create_chapter_combinations(&new_volume_ids, chapter_ids)?;
        info!("Finished extending database.");
        self.generate_manifest()?;
        Ok(())
    }
    /// Identifies missing database files and creates them
    /// by transforming unincorporated raw data.
    ///
    /// Files are considered missing if they are present in the manifest and
    /// absent in the file system.
    pub fn repair_from_raw(&self) -> Result<()> {
        let audit = self.check_completeness()?;
        let missing_chapters = audit.missing_chapters()?;
        if missing_chapters.is_empty() {
            info!("Database is complete. No repairs needed.");
            return Ok(());
        }
        info!(
            "{} Chapter(s) are missing and will be created from raw data.",
            missing_chapters.len()
        );
        self.create_specific_chapters(missing_chapters)?;
        info!("Finished rapairing database.");

        Ok(())
    }
    /// Creates every possible Chapter using the VolumeIds/ChapterIds provided.
    ///
    /// Every combination of is created.
    ///
    /// Used by self.full_transform() and self.extend().
    fn create_chapter_combinations(
        &self,
        volume_ids: &[T::AssociatedVolumeId],
        chapter_ids: &[T::AssociatedChapterId],
    ) -> Result<()> {
        info!(
            "{} VolumeIds, each with {} ChapterIds.",
            volume_ids.len(),
            chapter_ids.len()
        );
        let mut ids: Vec<(&T::AssociatedVolumeId, &T::AssociatedChapterId)> = vec![];
        for v in volume_ids {
            for c in chapter_ids {
                ids.push((v, c))
            }
        }
        self.create_specific_chapters(&ids)?;
        Ok(())
    }
    /// Creates specific Chapters using the VolumeIds/ChapterIds provided.
    ///
    /// Used by self.repair() and indirectly by self.full_transform() and self.extend().
    fn create_specific_chapters(
        &self,
        ids: &[(&T::AssociatedVolumeId, &T::AssociatedChapterId)],
    ) -> Result<()> {
        let total_chapters = ids.len() as u32;
        info!("{} total Chapters.", total_chapters);
        let count = Arc::new(Mutex::new(0_u32));

        ids.par_iter().for_each(|(volume_id, chapter_id)| {
            self.create_chapter(volume_id, chapter_id);
            log_count(
                count.clone(),
                total_chapters,
                "Finished checking/creating chapter",
                100,
            );
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
    /// Checks the database for completeness with respect the manifest file
    /// present.
    ///
    /// ## Algorithm
    ///
    /// - Check for missing Chapter directories (a user may only need a subset).
    /// - Of the present chapter directories, check volumes one at a time.
    ///     - If a volume is absent, record the reason (bad hash, no file)
    ///     - If a volume is absent across all chapter directories, then record the vol id
    ///     - Otherwise record the individual absent files.
    pub fn check_completeness(&self) -> Result<CompletenessAudit<T>> {
        let manifest = self.manifest()?;

        let mut audit = CompletenessAudit {
            absent_chapter_ids: vec![],
            absent_volume_ids: vec![],
            absent_individual_files: vec![],
        };
        // Check directories first.
        let present = self.chapters_present()?;
        for c in T::get_all_chapter_ids()? {
            if !present.contains(&c) {
                audit.absent_chapter_ids.push(c)
            }
        }
        // Check files.
        let latest_manifest_vol =
            T::AssociatedVolumeId::from_interface_id(manifest.latest_volume_identifier())?;
        let all_possible_volumes = latest_manifest_vol.all_prior()?;
        // VolumeIds with at least one valid file observed.
        let mut vols_seen: Vec<T::AssociatedVolumeId> = vec![];

        for m in manifest.cids()? {
            if audit.absent_chapter_ids.contains(&m.chapter_id) {
                // Skip file if its directory is known to be absent by its ChapterId.
                continue;
            }
            // Try to read the file.
            let chap_dir = self.config.chapter_dir_path(&m.chapter_id);
            let filename = T::AssociatedChapter::new_empty(&m.volume_id, &m.chapter_id).filename();
            let filepath = chap_dir.join(filename);

            // If it is absent, ::NoFile
            if !filepath.exists() {
                let abs = AbsentFile::NoFile(m.volume_id, m.chapter_id);
                audit.absent_individual_files.push(abs);
                continue;
            }

            // If it is wrong, ::DifferentHash
            let bytes = fs::read(filepath)?;
            let file_cid = cid_v0_string_from_bytes(&bytes)?;
            if m.cid != file_cid {
                let abs = AbsentFile::DifferentHash(m.volume_id, m.chapter_id);
                audit.absent_individual_files.push(abs);
                continue;
            }

            // If is is present, add to vols_seen (unless alread there).
            if !vols_seen.contains(&m.volume_id) {
                // Record all volumes that are seen at least once.
                vols_seen.push(m.volume_id)
            }
        }

        for v in all_possible_volumes {
            if !vols_seen.contains(&v) {
                audit.absent_volume_ids.push(v)
            }
        }

        Ok(audit)
    }
    /// Gets the ChapterIds of the Chapter directories that exist in the file system.
    ///
    /// Does not check if the directories are empty.
    fn chapters_present(&self) -> Result<Vec<T::AssociatedChapterId>> {
        let chapter_dirs = fs::read_dir(&self.config.data_dir).with_context(|| {
            format!("Couldn't read data directory {:?}.", &self.config.data_dir)
        })?;
        let mut chapters_present: Vec<T::AssociatedChapterId> = vec![];
        for chapter_dir in chapter_dirs {
            // Obtain ChapterId from directory name.
            let dir = chapter_dir?.path();
            let chap_id = T::AssociatedChapterId::from_chapter_directory(&dir)?;
            chapters_present.push(chap_id);
        }
        Ok(chapters_present)
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
            chapter_id,
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
    pub fn find(&self, raw_record_key: &str) -> Result<Vec<T::AssociatedRecordValue>> {
        let target_record_key = T::raw_key_as_record_key(raw_record_key)?;
        let chapter_id = T::record_key_to_chapter_id(&target_record_key)?;
        let chap_dir = self.config.chapter_dir_path(&chapter_id);
        // Read each file and collect matching Values
        let files = fs::read_dir(&chap_dir)
            .with_context(|| format!("Failed to read dir {:?}", chap_dir))?;
        let mut matching: Vec<T::AssociatedRecordValue> = vec![];
        for filename in files {
            let path = filename?.path();
            debug!("Reading file: {:?}", path);
            let bytes =
                fs::read(&path).with_context(|| format!("Failed to read file from {:?}", path))?;
            let chapter = <T::AssociatedChapter>::from_file(bytes)
                .with_context(|| format!("Failed to read/decode file: {:?}", path))?;
            let records = chapter.records();
            for r in records {
                let key = r.key();
                if key == &target_record_key {
                    let val = r.value().clone();
                    matching.push(val)
                }
            }
        }
        Ok(matching)
    }
    pub fn manifest(&self) -> Result<T::AssociatedManifest> {
        let path = self.config.manifest_file_path()?;
        let str = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read manifest: {:?}", &path))?;
        let manifest: T::AssociatedManifest = serde_json::from_str(&str)?;
        Ok(manifest)
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
        warn!("TODO: Manifest should be downloaded by an end user, not sourced locally.");

        let mut relevant_chapter_ids: Vec<T::AssociatedChapterId> = vec![];
        for k in keys {
            let record_key = T::raw_key_as_record_key(k)?;
            let chapter_id = T::record_key_to_chapter_id(&record_key)?;
            relevant_chapter_ids.push(chapter_id);
        }
        let manifest = self.manifest()?;
        let mut tasks: Vec<DownloadTask> = vec![];
        for m in manifest.cids()? {
            if relevant_chapter_ids.contains(&m.chapter_id) {
                let url = Url::parse(gateway)?.join(&m.cid)?;
                let dest_dir = self.config.chapter_dir_path(&m.chapter_id);
                let filename =
                    T::AssociatedChapter::new_empty(&m.volume_id, &m.chapter_id).filename();
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
    /**
    Obtains the sample data for the database.

    This includes processed (TODD-compliant) samples and raw samples
    that can be used to create processed samples.

    Samples may be in the cross-platform path (Directories crate),
    the local folder (if repo is cloned from GH) or may need
    to be obtained from a custom source. This method tries each in that
    order.

    The processed samples may need to be created from the raw samples, which
    can be slow.

    ## Example
    ```
    # use anyhow::Result;
    # use min_know::{
    #    config::{address_appearance_index::Network, choices::{DataKind, DirNature}},
    #    database::types::Todd,
    #    specs::address_appearance_index::AAISpec,
    # };
    let data_kind = DataKind::AddressAppearanceIndex(Network::default());
    let db: Todd<AAISpec> = Todd::init(data_kind, DirNature::Sample)?;
    db.get_sample_data()?;
    # Ok::<(), anyhow::Error>(())
    ```
    */
    pub fn get_sample_data(&self) -> Result<()> {
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
        let local_example_dir_raw = self.config.local_sample_raw_source();
        let raw_sample_filenames = T::AssociatedSampleObtainer::raw_sample_filenames();

        if raw_source_dir.contains_files(&raw_sample_filenames)? {
            info!("Checking raw sample files: already present.");
            return Ok(());
        }

        if local_example_dir_raw.contains_files(&raw_sample_filenames)? {
            info!("Raw sample files found in local repository: moving to samples directory.");
            local_example_dir_raw.copy_into_recursive(raw_source_dir)?;
        } else {
            info!("Raw samples not found: downloading.");
            T::AssociatedSampleObtainer::get_raw_samples(raw_source_dir)?
        }
        Ok(())
    }
    /// Ensures that the processed samples are either present or obtained.
    ///
    /// First looks in the expected location, then looks in the local
    /// directory (and copies if present), then attempts to processes from raw
    /// data.
    fn handle_database_samples(&self) -> Result<()> {
        let example_dir_processed = self.config.local_sample_data_dir();

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
            let chap_dir = self.config.data_dir.join(dirname);
            // Detect if any of the sample files are missing.
            if !chap_dir.contains_files(filenames)? {
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
            let chap_dir = example_dir_processed.join(dirname);
            // Detect if any of the sample files are missing.
            if !chap_dir.contains_files(filenames)? {
                local_data_dir_complete = false
            }
        }
        if local_data_dir_complete {
            info!("Local directory has sample files: copying to samples directory.");
            for (dirname, _filenames) in &dirnames_and_files {
                let src_chap_dir = example_dir_processed.join(dirname);
                let dest_chap_dir = self.config.data_dir.join(dirname);
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

/// A file that is in a given manifest, but not available for some reason.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AbsentFile<T: DataSpec> {
    DifferentHash(T::AssociatedVolumeId, T::AssociatedChapterId),
    NoFile(T::AssociatedVolumeId, T::AssociatedChapterId),
}

/// The status of the local database completeness with respect to a manifest.
///
/// Files are considered absent if they are present in the manifest and
/// absent in the file system.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompletenessAudit<T: DataSpec> {
    /// VolumeIds in the Manifest that do not appear anywhere in the file system.
    pub absent_volume_ids: Vec<T::AssociatedVolumeId>,
    /// ChapterIds in the Manifest that do not appear anywhere in the file system.
    pub absent_chapter_ids: Vec<T::AssociatedChapterId>,
    /// Files in the manifest that do not appear in the file system.
    ///
    /// Excludes files that are absent as part of a missing set of ChapterId/VolumeId.
    pub absent_individual_files: Vec<AbsentFile<T>>,
}

impl<T: DataSpec> CompletenessAudit<T> {
    fn missing_chapters(&self) -> Result<&[(&T::AssociatedVolumeId, &T::AssociatedChapterId)]> {
        todo!()
    }
}

impl<T: DataSpec> std::fmt::Display for CompletenessAudit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} missing ChapterIds, {} missing VolumeIds and {} missing individual files",
            self.absent_volume_ids.len(),
            self.absent_chapter_ids.len(),
            self.absent_individual_files.len()
        )
    }
}

/// Logs a counter with a message every time the count reaches a threshold.
fn log_count(count: Arc<Mutex<u32>>, total: u32, message: &str, threshold: u32) {
    let mut c = count.lock().unwrap();
    *c += 1;
    if *c % threshold == 0 {
        info!("{} {} of {}", message, c, total)
    }
}

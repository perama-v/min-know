use anyhow::Result;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    config::dirs::{ConfigStruct, DataKind, DirNature},
    specs::types::DataSpec,
};

/// The definition for the entire new database.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Todd<T: DataSpec> {
    pub chapters: Vec<Chapter<T>>,
    pub config: ConfigStruct,
}

/// Implement generic methods common to all databases.
impl<T: DataSpec> Todd<T> {
    pub fn new(specification: DataKind, directories: DirNature) -> Result<Self> {
        // Use the spec to then get the DataConfig.
        let config = directories.to_config(specification);

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
    ) -> Result<Chapter<T>> {
        // let mut elems: Vec<T::AssociatedRecordValue> = vec![];
        let mut vals: Vec<Record<T>> = vec![];
        let source_data: Vec<(&str, V)> = self.raw_pairs();
        for (raw_key, raw_val) in source_data {
            let record_key = T::raw_key_as_record_key(raw_key)?;
            if T::record_key_matches_chapter(&record_key, &vol, &chapter) {
                let record_value = T::raw_value_as_record_value(raw_val);
                todo!("Record value is collection of things.");
                let rec = Record{ key: record_key, value: record_value };
                vals.push(record_value)
            }
        }
        let mut chapter = Chapter::new(vol.clone(), chapter.clone());

        chapter.elems = todo!("Populate chapter");
        Ok(chapter)
    }
    pub fn raw_pairs<V>(&self) -> Vec<(&str, V)> {
        // A vector of generic key-value pairs.
        // E.g., (address, appearances) or (address, ABIs)
        todo!()
    }
    pub fn save_chapter(&self, c: Chapter<T>) {}
    /// Obtains the RecordValues that match a particular RecordKey
    ///
    /// Each Chapter contains Records with key-value pairs. This function
    /// aggregates values from all relevant Records (across different Chapters).
    pub fn values_matching(&self, raw_record_key: &str) -> Result<Vec<T::AssociatedRecordValue>> {
        let record_key = T::raw_key_as_record_key(raw_record_key)?;
        let chapter_id = T::record_key_to_chapter_id(record_key)?;
        let dir = self.config.source_root_dir();
        // Read each file and collect matching Values
        todo!("Port discover::single_address()")
    }
}

/// The distributable part of the database that is obtained from peers.
///
/// Internally consists of smaller useful pieces of data called RecordValues.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Chapter<T: DataSpec> {
    pub chapter_id: T::AssociatedChapterId,
    pub volume_id: T::AssociatedVolumeId,
    pub elems: Vec<Record<T>>,
}

impl<T: DataSpec> Chapter<T> {
    pub fn new(vol: T::AssociatedVolumeId, chapter: T::AssociatedChapterId) -> Self {
        Chapter {
            chapter_id: todo!(),
            volume_id: todo!(),
            elems: todo!(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Record<T: DataSpec> {
    pub key: T::AssociatedRecordKey,
    pub value: Vec<T::AssociatedRecordValue>,
}
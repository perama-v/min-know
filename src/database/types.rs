use anyhow::Result;
use std::{collections::BTreeMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{
    config::types::{DataConfigMethods, DestinationDataPath, DirLocation, SourceDataPath},
    specs::types::{DataSpec, SpecId},
};

/// The definition for the entire new database.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Todd<T: DataSpec, U: DataConfigMethods> {
    pub chapters: Vec<Chapter<T>>,
    pub config: U,
}

/// Implement generic methods common to all databases.
impl<T: DataSpec, U: DataConfigMethods> Todd<T, U> {
    pub fn new<V, W>(specification: SpecId, directories: DirLocation<V, W>) -> Result<Self>
    where
        V: SourceDataPath,
        W: DestinationDataPath,
    {
        // Use the spec to then get the DataConfig.
        let config: U = directories.to_config(specification)?;

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
        let mut elems: Vec<T::AssociatedRecordValue> = vec![];
        let source_data: Vec<(&str, V)> = self.raw_pairs();
        for (raw_key, raw_val) in source_data {
            let record_key = T::raw_key_as_record_key(raw_key)?;
            if T::record_key_matches_chapter(&record_key, vol, chapter) {
                let record_value = T::raw_value_as_record_value(raw_val);
                elems.push(record_value)
            }
        }
        let mut chapter = Chapter::new(vol.clone(), chapter.clone());
        chapter.elems = elems;
        Ok(chapter)
    }
    pub fn raw_pairs<V>(&self) -> Vec<(&str, V)> {
        // A vector of generic key-value pairs.
        // E.g., (address, appearances) or (address, ABIs)
        todo!()
    }
    pub fn save_chapter(&self, c: Chapter<T>) {}
    /// Obtains the values that match a particular key
    pub fn read_record_key(&self, raw_record_key: &str) -> Result<T::AssociatedRecordValue> {
        let record_key = T::raw_key_as_record_key(raw_record_key)?;
        // Get ChapterId
        let chapter_id = T::record_key_to_chapter_id(record_key)?;
        // Get Chapter Files

        // Read each file and collect matching Values
        todo!()
    }
}

/// The distributable part of the database that is obtained from peers.
///
/// Internally consists of smaller useful pieces of data called RecordValues.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub struct Chapter<T: DataSpec> {
    pub chapter_id: T::AssociatedChapterId,
    pub volume_id: T::AssociatedVolumeId,
    pub elems: Vec<T::AssociatedRecordValue>,
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

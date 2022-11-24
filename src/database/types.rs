use anyhow::Result;
use std::{collections::BTreeMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::specs::types::DataSpec;

/// The definition for the entire new database.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Todd<T: DataSpec> {
    pub units: Vec<Unit<T>>,
}

/// The distributable part of the database that is obtained from peers.
///
/// Internally consists of smaller useful pieces of data called RecordValues.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub struct Unit<T: DataSpec> {
    pub chapter_id: T::AssociatedChapterId,
    pub volume_id: T::AssociatedVolumeId,
    pub elems: Vec<T::AssociatedRecordValue>,
}

impl<T: DataSpec> Unit<T> {
    pub fn new(vol: T::AssociatedVolumeId, chapter: T::AssociatedChapterId) -> Self {
        Unit {
            chapter_id: todo!(),
            volume_id: todo!(),
            elems: todo!(),
        }
    }
}

/// Implement generic methods common to all databases.
impl<T: DataSpec> Todd<T> {
    pub fn new() -> Self {
        Self { units: vec![] }
    }
    // Creates new and complete todd.
    pub fn full_transform<U>(&mut self) -> Result<()> {
        let chapts = T::get_all_chapter_ids();
        let vols = T::get_all_volume_ids();
        for chapter in &chapts {
            for vol in &vols {
                let unit = self.get_one_unit::<U>(vol, chapter)?;
                self.save_unit(unit);
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
    /// Prepares the mininum distributable Unit
    pub fn get_one_unit<U>(
        &self,
        vol: &T::AssociatedVolumeId,
        chapter: &T::AssociatedChapterId,
    ) -> Result<Unit<T>> {
        let mut elems: Vec<T::AssociatedRecordValue> = vec![];
        let source_data: Vec<(&str, U)> = self.raw_pairs();
        for (raw_key, raw_val) in source_data {
            let record_key = T::raw_key_as_record_key(raw_key)?;
            if T::record_key_matches_unit(&record_key, vol, chapter) {
                let record_value = T::raw_value_as_record_value(raw_val);
                elems.push(record_value)
            }
        }
        let mut unit = Unit::new(vol.clone(), chapter.clone());
        unit.elems = elems;
        Ok(unit)
    }
    pub fn raw_pairs<V>(&self) -> Vec<(&str, V)> {
        // A vector of generic key-value pairs.
        // E.g., (address, appearances) or (address, ABIs)
        todo!()
    }
    pub fn save_unit(&self, u: Unit<T>) {}
    /// Obtains the values that match a particular key
    ///
    ///
    pub fn read_record_key(&self, raw_record_key: &str) -> T::AssociatedRecordValue {
        let record_key = T::raw_key_as_record_key(raw_record_key);
        todo!("Use record_key to find appropriate Unit & RecordValue.")
    }
}

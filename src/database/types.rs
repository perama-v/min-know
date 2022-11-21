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
/// Internally consists of smaller useful pieces of data called Elements.
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Hash, Deserialize)]
pub struct Unit<T: DataSpec> {
    pub chapter_id: T::AssociatedChapterId,
    pub volume_id: T::AssociatedVolumeId,
    pub elems: Vec<T::AssociatedElement>,
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
    pub fn full_transform<V>(&mut self) -> Result<()> {
        let chapts = T::get_all_chapter_ids();
        let vols = T::get_all_volume_ids();
        for chapter in &chapts {
            for vol in &vols {
                let unit = self.get_one_unit::<V>(vol, chapter)?;
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
    pub fn get_one_unit<V>(
        &self,
        vol: &T::AssociatedVolumeId,
        chapter: &T::AssociatedChapterId,
    ) -> Result<Unit<T>> {
        let mut elems: Vec<T::AssociatedElement> = vec![];
        let source_data: Vec<(&str, V)> = self.raw_pairs();
        for (raw_key, raw_val) in source_data {
            let query = T::raw_key_as_query(raw_key)?;
            if T::query_matches_unit(&query, vol, chapter) {
                let element = T::raw_value_as_element(raw_val);
                elems.push(element)
            }
        }
        let mut unit = Unit::new(vol.clone(), chapter.clone());
        unit.elems = elems;
        Ok(unit)
    }
    pub fn raw_pairs<U, V>(&self) -> Vec<(U, V)> {
        // A vector of generic key-value pairs.
        // E.g., (address, appearances) or (address, ABIs)
        todo!()
    }
    pub fn save_unit(&self, u: Unit<T>) {}
    pub fn read_query(&self, raw_query: &str) -> T::AssociatedElement {
        let query = T::raw_key_as_query(raw_query);
        todo!("Use query to find appropriate Unit & Element.")
    }
}

use std::{
    fs::{self, read_dir},
    path::Path,
};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use ssz_rs::List;

use crate::{
    parameters::nametags::ENTRIES_PER_VOLUME,
    specs::nametags::{
        NameTagsChapter, NameTagsChapterId, NameTagsRecord, NameTagsRecordKey, NameTagsRecordValue,
        NameTagsSpec, NameTagsVolumeId,
    },
};

use super::traits::ExtractorMethods;

/// Strongly typed parser for the JSON data in the raw (unprocessed data).
#[derive(Serialize, Deserialize)]
pub struct RawValue {
    /// Raw data only has one name per address.
    ///
    /// Note that the processed data can theoretically hold more than one.
    pub name: Option<String>,
    /// Raw data has multiple tags per address.
    pub tags: Option<Vec<String>>,
}

impl RawValue {
    /// Creates a record value from from raw data.
    fn into_record_value(self) -> NameTagsRecordValue {
        // Allow for 0, 1 or more names.
        let names = match self.name {
            Some(n) => vec![n],
            None => vec![],
        };
        // Allow for 0, 1 or more tags.
        let tags = match self.tags {
            Some(t) => t,
            None => vec![],
        };
        NameTagsRecordValue::from_strings(names, tags)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsExtractor;

impl ExtractorMethods<NameTagsSpec> for NameTagsExtractor {
    fn chapter_from_raw(
        chapter_id: &NameTagsChapterId,
        volume_id: &NameTagsVolumeId,
        source_dir: &Path,
    ) -> Result<Option<NameTagsChapter>> {
        let Ok(dir) = fs::read_dir(source_dir) else {
            bail!("Couldn't read dir {}", source_dir.display())};
        // Get appropriate range and appropriate files in that range.
        let mut records: Vec<NameTagsRecord> = vec![];
        // Files are ordered deterministically (but not lexicographically),
        // so picking out the right files by index is ok.
        let relevant_files = dir
            .skip(volume_id.first_address as usize)
            .take(ENTRIES_PER_VOLUME)
            .collect::<Result<Vec<_>, _>>()?;

        for file in relevant_files {
            let name = file.file_name();
            let Some(address) = name.to_str() else {
                bail!("Couldn't read filename: {}", file.path().display())};
            // '0xabcd' -> 'ab'
            let candidate: String = address.to_string().chars().skip(2).take(2).collect();
            if chapter_id.matches(&candidate) {
                // Make NameTagsRecord
                let contents = fs::read(file.path())?;
                let data: RawValue = serde_json::from_slice(&contents)?;
                let record = NameTagsRecord {
                    key: NameTagsRecordKey::from_address(address)?,
                    value: data.into_record_value(),
                };
                records.push(record);
            }
        }
        if records.is_empty() {
            return Ok(None);
        }
        // Make and return NameTagsChapter{}
        Ok(Some(NameTagsChapter {
            chapter_id: chapter_id.clone(),
            volume_id: volume_id.clone(),
            records: List::from_iter(records),
        }))
    }

    fn latest_possible_volume(source_dir: &Path) -> Result<NameTagsVolumeId> {
        let Ok(dir) = read_dir(source_dir) else {bail!("Can't read: {}", source_dir.display())};
        let count = dir.count() as u32;
        let first_address = first_inside_last(count, ENTRIES_PER_VOLUME as u32)?;
        Ok(NameTagsVolumeId { first_address })
    }
}

/// Gets the global index of the first address in the last volume.
fn first_inside_last(count: u32, capacity: u32) -> Result<u32> {
    if count < capacity {
        bail!(
            "Not enough data to make the first Volume. (need: {}, have: {})",
            capacity,
            count
        )
    }
    let complete_vols = count / capacity;
    let first_address = capacity * (complete_vols - 1);
    Ok(first_address)
}

#[test]
fn latest_in_sample() {
    assert!(first_inside_last(999, 1000).is_err());
    assert_eq!(first_inside_last(1000, 1000).unwrap(), 0);
    assert_eq!(first_inside_last(1001, 1000).unwrap(), 0);
    assert_eq!(first_inside_last(1999, 1000).unwrap(), 0);
    assert_eq!(first_inside_last(2000, 1000).unwrap(), 1000);
    assert_eq!(first_inside_last(2001, 1000).unwrap(), 1000);
}

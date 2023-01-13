use anyhow::{bail, Result};
use ssz_rs::List;
use std::{
    fs::{self, read_dir},
    path::Path,
};

use crate::{
    parameters::signatures::SIGNATURES_PER_VOLUME,
    specs::signatures::{
        SignaturesChapter, SignaturesChapterId, SignaturesRecord, SignaturesRecordKey,
        SignaturesRecordValue, SignaturesSpec, SignaturesVolumeId, Text,
    },
};

use super::traits::ExtractorMethods;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignaturesExtractor {}

impl ExtractorMethods<SignaturesSpec> for SignaturesExtractor {
    fn chapter_from_raw(
        chapter_id: &SignaturesChapterId,
        volume_id: &SignaturesVolumeId,
        source_dir: &Path,
    ) -> Result<Option<SignaturesChapter>> {
        let Ok(dir) = fs::read_dir(source_dir) else {
            bail!("Couldn't read dir {}", source_dir.display())};
        // Get appropriate range and appropriate files in that range.
        let mut records: Vec<SignaturesRecord> = vec![];
        // Files are ordered deterministically (but not lexicographically),
        // so picking out the right files by index is ok.
        let relevant_files = dir
            .skip(volume_id.first_signature as usize)
            .take(SIGNATURES_PER_VOLUME)
            .collect::<Result<Vec<_>, _>>()?;

        for file in relevant_files {
            let name = file.file_name();
            let Some(signature) = name.to_str() else {
                bail!("Couldn't read filename: {}", file.path().display())};
            // 'abcdef01' -> 'abcdef01' and 'abcdef01234567...' -> 'abcdef01'
            let candidate: String = signature.to_string().chars().take(8).collect();

            if chapter_id.matches(&candidate) {
                // Make SignaturesRecord
                let contents = fs::read_to_string(file.path())?;
                // Format if collisions: "<text>;<text>;<text>"
                let texts: Vec<Text> = contents.split(';').map(Text::from_string).collect();

                let record = SignaturesRecord {
                    key: SignaturesRecordKey::from_signature(signature)?,
                    value: SignaturesRecordValue {
                        texts: List::from_iter(texts),
                    },
                };
                records.push(record);
            }
        }
        if records.is_empty() {
            return Ok(None);
        }
        // Make and return SignaturesChapter{}
        Ok(Some(SignaturesChapter {
            chapter_id: chapter_id.clone(),
            volume_id: volume_id.clone(),
            records: List::from_iter(records),
        }))
    }

    fn latest_possible_volume(source_dir: &Path) -> Result<SignaturesVolumeId> {
        let Ok(dir) = read_dir(source_dir) else {bail!("Can't read: {}", source_dir.display())};
        let count = dir.count() as u32;
        let first_signature = first_inside_last(count, SIGNATURES_PER_VOLUME as u32)?;
        Ok(SignaturesVolumeId { first_signature })
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

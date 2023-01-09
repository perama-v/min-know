use std::{fs::read_dir, path::Path};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::{
    parameters::nametags::ENTRIES_PER_VOLUME,
    specs::nametags::{NameTagsChapter, NameTagsChapterId, NameTagsSpec, NameTagsVolumeId},
};

use super::traits::ExtractorMethods;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NameTagsExtractor;

impl ExtractorMethods<NameTagsSpec> for NameTagsExtractor {
    fn chapter_from_raw(
        chapter_id: &NameTagsChapterId,
        volume_id: &NameTagsVolumeId,
        source_dir: &Path,
    ) -> Result<Option<NameTagsChapter>> {
        todo!()
    }

    fn latest_possible_volume(source_dir: &Path) -> Result<NameTagsVolumeId> {
        let Ok(dir) = read_dir(source_dir) else {bail!("Can't read: {}", source_dir.display())};
        let count = dir.count() as u32;
        let first_address = first_inside_last(count, ENTRIES_PER_VOLUME)?;
        Ok(NameTagsVolumeId { first_address })
    }
}

/// Gets the global index of the first address in the last volume.
fn first_inside_last(count: u32, capacity: u32) -> Result<u32> {
    if count < capacity {
        bail!("Not enough data to make the first Volume. (need: {}, have: {})",
        capacity, count)
    }
    let complete_vols = count / capacity;
    let first_address = (capacity * (complete_vols - 1)) as u32;
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
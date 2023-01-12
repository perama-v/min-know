use std::fmt::Display;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::specs::{
    address_appearance_index::{AAIChapterId, AAISpec, AAIVolumeId},
    traits::{ChapterIdMethods, ManifestCids, ManifestMethods, VolumeIdMethods},
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AAIManifest {
    pub spec_version: String,
    pub schemas: String,
    pub database_interface_id: String,
    pub latest_volume_identifier: String,
    pub chapter_cids: Vec<AAIManifestChapter>,
}

impl ManifestMethods<AAISpec> for AAIManifest {
    fn spec_version(&self) -> &str {
        &self.spec_version
    }

    fn set_spec_version(&mut self, version: String) {
        self.spec_version = version
    }

    fn schemas(&self) -> &str {
        &self.schemas
    }

    fn set_schemas(&mut self, schemas: String) {
        self.schemas = schemas
    }

    fn database_interface_id(&self) -> &str {
        &self.database_interface_id
    }

    fn set_database_interface_id(&mut self, id: String) {
        self.database_interface_id = id;
    }

    fn latest_volume_identifier(&self) -> &str {
        &self.latest_volume_identifier
    }

    fn set_latest_volume_identifier(&mut self, volume_interface_id: String) {
        self.latest_volume_identifier = volume_interface_id
    }

    fn cids(&self) -> Result<Vec<ManifestCids<AAISpec>>> {
        let mut result: Vec<ManifestCids<AAISpec>> = vec![];
        for chapter in &self.chapter_cids {
            let volume_id = AAIVolumeId::from_interface_id(&chapter.volume_interface_id)?;
            let chapter_id = AAIChapterId::from_interface_id(&chapter.chapter_interface_id)?;
            result.push(ManifestCids {
                cid: chapter.cid_v0.clone(),
                volume_id,
                chapter_id,
            })
        }
        Ok(result)
    }

    fn set_cids<C>(&mut self, cids: &[(C, AAIVolumeId, AAIChapterId)])
    where
        C: AsRef<str> + Display,
    {
        for (cid, volume_id, chapter_id) in cids {
            let chapter = AAIManifestChapter {
                volume_interface_id: volume_id.interface_id(),
                chapter_interface_id: chapter_id.interface_id(),
                cid_v0: cid.to_string(),
            };
            self.chapter_cids.push(chapter)
        }
        // Sort by VolumeId, then by ChapterId for ties.
        self.chapter_cids.sort_by(|a, b| {
            a.volume_interface_id
                .cmp(&b.volume_interface_id)
                .then(a.chapter_interface_id.cmp(&b.chapter_interface_id))
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AAIManifestChapter {
    pub volume_interface_id: String,
    pub chapter_interface_id: String,
    pub cid_v0: String,
}

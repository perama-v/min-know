use std::path::PathBuf;

use anyhow::Result;

use super::types::*;

pub enum Source {
    Sample,
    Default,
    Custom(PathBuf),
}

impl SourceDataPath for Source {
    fn root_dir(&self) -> anyhow::Result<std::path::PathBuf> {
        todo!()
    }
}

pub enum Destination {
    Sample,
    Default,
    Custom(PathBuf),
}

impl DestinationDataPath for Destination {
    fn root_dir(&self) -> anyhow::Result<std::path::PathBuf> {
        todo!()
    }

    fn chapters_dir(&self) -> anyhow::Result<std::path::PathBuf> {
        todo!()
    }

    fn manifest_file(&self) -> anyhow::Result<std::path::PathBuf> {
        todo!()
    }

    fn chapter_path(&self) -> anyhow::Result<std::path::PathBuf> {
        todo!()
    }

    fn latest_volume(&self) -> anyhow::Result<crate::spec::VolumeIdentifier> {
        todo!()
    }
}

pub struct Name {}

impl DataName for Name {
    fn name() -> String {
        String::from("Address Appearance Index")
    }
}

pub struct AdApConfig {
    source: Source,
    destination: Destination,
}

impl AdApConfig {
    pub fn new<T, U>(dir_location: &DirLocation<T, U>) -> Result<AdApConfig>
    where
        T: SourceDataPath,
        U: DestinationDataPath,
    {
        let config = match dir_location {
            DirLocation::Sample => AdApConfig {
                source: Source::Sample,
                destination: Destination::Sample,
            },
            DirLocation::Default => AdApConfig {
                source: Source::Default,
                destination: Destination::Default,
            },
            DirLocation::Custom(x, y) => {
                let custom_source = x.root_dir()?;
                let custom_dest = y.root_dir()?;
                AdApConfig {
                    source: Source::Custom(custom_source),
                    destination: Destination::Custom(custom_dest),
                }
            }
        };
        Ok(config)
    }
}

impl DataConfigMethods for AdApConfig {
    type Source = Source;
    type Destination = Destination;
    type Name = Name;
}

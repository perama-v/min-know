use super::types::*;

pub struct Source {}

impl SourceDataPath for Source {}

pub struct Destination {}

impl DestinationDataPath for Destination {}

pub struct Name {}

impl DataName for Name {
    fn name() -> String {
        String::from("Address Appearance Index")
    }
}

pub struct Config {}

impl DatabaseConfig for Config {
    type Source = Source;
    type Destination = Destination;
    type Name = Name;
}

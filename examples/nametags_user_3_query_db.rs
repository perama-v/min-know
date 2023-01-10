use std::env;

use anyhow::Result;

use min_know::{
    config::{
        address_appearance_index::Network,
        choices::{DataKind, DirNature},
    },
    database::types::Todd,
    specs::{address_appearance_index::{AAIAppearanceTx, AAISpec}, nametags::{Name, Tag, NameTagsSpec}},
};

/// Uses local index data to extract transaction identifiers important for a given address.
fn main() -> Result<()> {
    // For full error backtraces with anyhow.
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let data_kind = DataKind::NameTags;
    let db: Todd<NameTagsSpec> = Todd::init(data_kind, DirNature::Sample)?;
    let address = "0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae";
    /*
        names: ["EthDev"]
        tags: ["ethereum-foundation"]
    */

    //let address = "0000000000000000000000000000000000000000";

    /*
        names: ["Null Address: 0x000…000"]
        tags: ["burn", "genesis"]
    */
    let values = db.find(address)?;
    let mut names = vec![];
    let mut tags = vec![];
    for v in values {
        names.extend(v.names_as_strings()?);
        tags.extend(v.tags_as_strings()?);
    }
    println!("names: {:?}\ntags: {:?}", names, tags);
    Ok(())
}

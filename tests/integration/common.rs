use min_know::{
    config::{
        address_appearance_index::Network,
        choices::{DataKind, DirNature},
    },
    database::types::Todd,
    specs::{address_appearance_index::AAISpec, nametags::NameTagsSpec},
};

pub fn aai_db() -> Todd<AAISpec> {
    let data_kind = DataKind::AddressAppearanceIndex(Network::default());
    let db: Todd<AAISpec> = Todd::init(data_kind, DirNature::Sample).unwrap();
    db
}

pub fn nametags_db() -> Todd<NameTagsSpec> {
    let data_kind = DataKind::NameTags;
    let db: Todd<NameTagsSpec> = Todd::init(data_kind, DirNature::Sample).unwrap();
    db
}

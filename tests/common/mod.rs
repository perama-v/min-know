use min_know::{
    config::dirs::{DataKind, DirNature},
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
};

pub fn aai_db() -> Todd<AAISpec> {
    let mut db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample).unwrap();
    db
}

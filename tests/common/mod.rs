use min_know::{
    config::choices::{DataKind, DirNature},
    database::types::Todd,
    specs::address_appearance_index::AAISpec,
};

pub fn aai_db() -> Todd<AAISpec> {
    let db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample).unwrap();
    db
}

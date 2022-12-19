use min_know::{database::types::Todd, specs::{address_appearance_index::{AAISpec}}, config::dirs::{DataKind, DirNature}};

pub fn aai_db() -> Todd<AAISpec> {
    let mut db: Todd<AAISpec> = Todd::init(DataKind::default(), DirNature::Sample).unwrap();
    db
}

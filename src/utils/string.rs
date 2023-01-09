/// Turns u32 into 000_000_000 formatted string.
pub fn num_as_triplet(number: u32) -> String {
    let mut name = format!("{:0>9}", number);
    for i in [6, 3] {
        name.insert(i, '_');
    }
    name
}

#[test]
fn triplet_splits_ok() {
    assert_eq!(num_as_triplet(4_010_302), String::from("004_010_302"));
}
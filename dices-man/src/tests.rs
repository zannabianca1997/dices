//! Test checking the well-formness of the manual

use crate::search;

#[test]
fn introduction_exist() {
    assert!(search("introduction").is_some())
}
#[test]
fn index_exist() {
    assert!(search("index").is_some())
}

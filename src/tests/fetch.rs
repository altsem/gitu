use super::*;

#[test]
fn fetch_from_elsewhere_prompt() {
    snapshot!(setup_clone!(), "fe");
}

#[test]
fn fetch_from_elsewhere() {
    snapshot!(setup_clone!(), "feorigin<enter>");
}

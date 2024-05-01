use super::*;

#[test]
fn fetch_from_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "fe");
}

#[test]
fn fetch_from_elsewhere() {
    snapshot!(TestContext::setup_clone(), "feorigin<enter>");
}

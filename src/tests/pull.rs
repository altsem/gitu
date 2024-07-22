use super::*;

#[test]
fn pull_from_elsewhere_prompt() {
    snapshot!(TestContext::setup_clone(), "Fe");
}

#[test]
fn pull_from_elsewhere() {
    snapshot!(TestContext::setup_clone(), "Feorigin<enter>");
}

pub(crate) fn str_vec(input: &[String]) -> Vec<&str> {
    input.iter().map(|s| s.as_ref()).collect::<Vec<_>>()
}

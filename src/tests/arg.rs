use crate::menu::arg::{self, Arg};

#[test]
fn value_as_u32() {
    let arg = Arg::new_arg("arg", "display", Some(|| 1u32), arg::positive_number);

    assert_eq!(arg.value_as::<String>(), None);
    assert_eq!(arg.value_as::<u32>(), Some(&1u32));
}

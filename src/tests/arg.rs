use crate::menu::arg::{self, Arg};

#[test]
fn flag_operations() {
    let mut arg = Arg::new_flag("--arg", "display", true);

    assert_eq!(arg.expects_value(), false);
    assert_eq!(arg.is_active(), true);
    assert_eq!(arg.default_as_string(), None);
    assert_eq!(arg.get_cli_token(), "--arg".to_string());

    arg.unset();
    assert_eq!(arg.expects_value(), false);
    assert_eq!(arg.is_active(), false);
    assert_eq!(arg.default_as_string(), None);
    assert_eq!(arg.get_cli_token(), "--arg".to_string());

    assert_eq!(arg.set("").ok(), Some(()));
    assert_eq!(arg.is_active(), true);
}

#[test]
fn arg_operations() {
    let mut arg = Arg::new_arg("--arg", "display", Some(|| 1u32), arg::positive_number);

    assert_eq!(arg.expects_value(), true);
    assert_eq!(arg.is_active(), true);
    assert_eq!(arg.default_as_string(), Some("1".to_string()));
    assert_eq!(arg.get_cli_token(), "--arg=1".to_string());

    arg.unset();
    assert_eq!(arg.expects_value(), true);
    assert_eq!(arg.is_active(), false);
    assert_eq!(arg.default_as_string(), Some("1".to_string()));
    assert_eq!(arg.get_cli_token(), "--arg".to_string());

    assert_eq!(arg.set("").ok(), None);
    assert_eq!(arg.is_active(), false);

    assert_eq!(arg.set("1").ok(), Some(()));
    assert_eq!(arg.is_active(), true);
}

#[test]
fn value_as_concrete_type() {
    let arg = Arg::new_arg("--arg", "display", Some(|| 1u32), arg::positive_number);

    assert_eq!(arg.value_as::<String>(), None);
    assert_eq!(arg.value_as::<u32>(), Some(&1u32));
}

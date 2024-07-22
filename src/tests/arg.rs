use crate::menu::arg::{self, Arg};

#[test]
fn flag_operations() {
    let mut arg = Arg::new_flag("--arg", "display", true);

    assert!(!arg.expects_value());
    assert!(arg.is_active());
    assert_eq!(arg.default_as_string(), None);
    assert_eq!(arg.get_cli_token(), "--arg".to_string());

    arg.unset();
    assert!(!arg.expects_value());
    assert!(!arg.is_active());
    assert_eq!(arg.default_as_string(), None);
    assert_eq!(arg.get_cli_token(), "--arg".to_string());

    assert_eq!(arg.set("").ok(), Some(()));
    assert!(arg.is_active());
}

#[test]
fn arg_operations() {
    let mut arg = Arg::new_arg("--arg", "display", Some(|| 1u32), arg::positive_number);

    assert!(arg.expects_value());
    assert!(arg.is_active());
    assert_eq!(arg.default_as_string(), Some("1".to_string()));
    assert_eq!(arg.get_cli_token(), "--arg=1".to_string());

    arg.unset();
    assert!(arg.expects_value());
    assert!(!arg.is_active());
    assert_eq!(arg.default_as_string(), Some("1".to_string()));
    assert_eq!(arg.get_cli_token(), "--arg".to_string());

    assert_eq!(arg.set("").ok(), None);
    assert!(!arg.is_active());

    assert_eq!(arg.set("1").ok(), Some(()));
    assert!(arg.is_active());
}

#[test]
fn value_as_concrete_type() {
    let arg = Arg::new_arg("--arg", "display", Some(|| 1u32), arg::positive_number);

    assert_eq!(arg.value_as::<String>(), None);
    assert_eq!(arg.value_as::<u32>(), Some(&1u32));
}

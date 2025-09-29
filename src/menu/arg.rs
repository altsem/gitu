use crate::{Res, error::Error};
use regex::Regex;

#[derive(Debug)]
pub(crate) struct Arg {
    pub arg: &'static str,
    pub display: &'static str,
    value: Box<dyn ArgValue>,
}

impl Arg {
    pub fn new_flag(arg: &'static str, display: &'static str, default: bool) -> Self {
        Arg {
            arg,
            display,
            value: Box::new(ArgBool { value: default }),
        }
    }

    pub fn new_arg<T>(
        arg: &'static str,
        display: &'static str,
        default: Option<fn() -> T>,
        parser: fn(&str) -> Res<T>,
    ) -> Self
    where
        T: std::fmt::Debug + std::fmt::Display + 'static,
    {
        Arg {
            arg,
            display,
            value: Box::new(ArgT::<T> {
                value: default.map(|fun| fun()),
                default,
                parser,
            }),
        }
    }

    pub fn is_active(&self) -> bool {
        self.value.is_set()
    }

    pub fn unset(&mut self) {
        self.value.unset()
    }

    pub fn expects_value(&self) -> bool {
        self.value.expects_value()
    }

    pub fn default_as_string(&self) -> Option<String> {
        self.value.default_as_string()
    }

    pub fn set(&mut self, value: &str) -> Res<()> {
        self.value.set(value)
    }

    pub fn value_as_string(&self) -> Option<String> {
        self.value.value_as_string()
    }

    pub fn value_as<T>(&self) -> Option<&T>
    where
        T: std::fmt::Debug + std::fmt::Display + 'static,
    {
        self.value.value_as_any().and_then(|x| x.downcast_ref())
    }

    pub fn get_cli_token(&self) -> String {
        match self.value_as_string() {
            Some(value) => format!("{}={}", self.arg, value),
            None => self.arg.to_string(),
        }
    }
}

trait ArgValue: std::fmt::Debug {
    fn is_set(&self) -> bool;
    fn unset(&mut self);
    fn expects_value(&self) -> bool;
    fn default_as_string(&self) -> Option<String>;
    fn set(&mut self, value: &str) -> Res<()>;
    fn value_as_string(&self) -> Option<String>;
    fn value_as_any(&self) -> Option<&dyn std::any::Any>;
}

#[derive(Debug)]
struct ArgBool {
    value: bool,
}

impl ArgValue for ArgBool {
    fn is_set(&self) -> bool {
        self.value
    }

    fn unset(&mut self) {
        self.value = false;
    }

    fn expects_value(&self) -> bool {
        false
    }

    fn default_as_string(&self) -> Option<String> {
        None
    }

    fn set(&mut self, _value: &str) -> Res<()> {
        self.value = true;
        Ok(())
    }

    fn value_as_string(&self) -> Option<String> {
        None
    }

    fn value_as_any(&self) -> Option<&dyn std::any::Any> {
        Some(&self.value)
    }
}

#[derive(Debug)]
struct ArgT<T> {
    value: Option<T>,
    default: Option<fn() -> T>,
    parser: fn(&str) -> Res<T>,
}

impl<T: std::fmt::Debug> ArgValue for ArgT<T>
where
    T: std::fmt::Display + 'static,
{
    fn is_set(&self) -> bool {
        self.value.is_some()
    }

    fn unset(&mut self) {
        self.value = None;
    }

    fn expects_value(&self) -> bool {
        true
    }

    fn default_as_string(&self) -> Option<String> {
        self.default.map(|x| x().to_string())
    }

    fn set(&mut self, value: &str) -> Res<()> {
        self.value = Some((self.parser)(value)?);
        Ok(())
    }

    fn value_as_string(&self) -> Option<String> {
        self.value.as_ref().map(|x| x.to_string())
    }

    fn value_as_any(&self) -> Option<&dyn std::any::Any> {
        self.value.as_ref().map(|x| x as &dyn std::any::Any)
    }
}

pub fn positive_number(s: &str) -> Res<u32> {
    let n = s.parse::<u32>().ok().unwrap_or(0);
    if n > 0 {
        return Ok(n);
    }

    Err(Error::ArgMustBePositiveNumber)
}

pub fn any_regex(s: &str) -> Res<Regex> {
    Regex::try_from(s).map_err(Error::ArgInvalidRegex)
}

#[cfg(test)]
mod tests {
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
}

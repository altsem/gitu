use crate::Res;
use dyn_clone::DynClone;

#[derive(Debug, Clone)]
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
        default: Option<T>,
        parser: fn(&str) -> Res<T>,
    ) -> Self
    where
        T: Clone + std::fmt::Debug + std::fmt::Display + 'static,
    {
        Arg {
            arg,
            display,
            value: Box::new(ArgT::<T> {
                value: default.clone(),
                default,
                parser,
            }),
        }
    }

    pub fn is_active(&self) -> bool {
        self.value.is_set()
    }

    pub fn unset(&mut self) -> () {
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
        T: Clone + std::fmt::Debug + std::fmt::Display + 'static,
    {
        self.value
            .value_as_any()
            .and_then(|x| x.downcast_ref::<T>())
    }

    pub fn get_cli_token(&self) -> String {
        match self.value_as_string() {
            Some(value) => format!("{}={}", self.arg, value),
            None => self.arg.to_string(),
        }
    }
}

trait ArgValueBase {
    fn is_set(&self) -> bool;
    fn unset(&mut self) -> ();
    fn expects_value(&self) -> bool;
    fn default_as_string(&self) -> Option<String>;
    fn set(&mut self, value: &str) -> Res<()>;
    fn value_as_string(&self) -> Option<String>;
    fn value_as_any(&self) -> Option<Box<&dyn std::any::Any>>;
}

trait ArgValue: ArgValueBase + core::fmt::Debug + DynClone {}

dyn_clone::clone_trait_object!(ArgValue);

#[derive(Debug, Clone)]
struct ArgBool {
    value: bool,
}

impl ArgValueBase for ArgBool {
    fn is_set(&self) -> bool {
        self.value
    }

    fn unset(&mut self) -> () {
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

    fn value_as_any(&self) -> Option<Box<&dyn std::any::Any>> {
        Some(Box::new(&self.value))
    }
}

impl ArgValue for ArgBool {}

#[derive(Debug, Clone)]
struct ArgT<T> {
    value: Option<T>,
    default: Option<T>,
    parser: fn(&str) -> Res<T>,
}

impl<T> ArgValueBase for ArgT<T>
where
    T: Clone + std::fmt::Display + 'static,
{
    fn is_set(&self) -> bool {
        self.value.is_some()
    }

    fn unset(&mut self) -> () {
        self.value = None;
    }

    fn expects_value(&self) -> bool {
        true
    }

    fn default_as_string(&self) -> Option<String> {
        self.default.clone().map(|x| x.to_string())
    }

    fn set(&mut self, value: &str) -> Res<()> {
        self.value = Some((self.parser)(&value)?);
        Ok(())
    }

    fn value_as_string(&self) -> Option<String> {
        self.value.clone().map(|x| x.to_string())
    }

    fn value_as_any(&self) -> Option<Box<&dyn std::any::Any>> {
        Some(Box::new(&self.value))
    }
}

impl<T> ArgValue for ArgT<T> where T: Clone + std::fmt::Debug + std::fmt::Display + 'static {}

pub fn positive_number(s: &str) -> Res<u32> {
    let n = s.parse::<u32>().ok().unwrap_or(0);
    if n > 0 {
        return Ok(n);
    }

    Err("Value must be a number greater than 0".into())
}

pub fn any_string(s: &str) -> Res<String> {
    Ok(s.to_string())
}

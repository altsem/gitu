use crate::Res;

#[derive(Debug, Clone)]
enum ArgValue {
    Bool(ArgBool),
    String(ArgT<String>),
    U32(ArgT<u32>),
}

#[derive(Debug, Clone)]
pub(crate) struct Arg {
    pub arg: &'static str,
    pub display: &'static str,
    value: ArgValue,
}

impl Arg {
    pub const fn new_flag(arg: &'static str, display: &'static str, default: bool) -> Self {
        Arg {
            arg,
            display,
            value: ArgValue::Bool(ArgBool { value: default }),
        }
    }

    pub const fn new_u32(
        arg: &'static str,
        display: &'static str,
        default: Option<u32>,
        parser: fn(&str) -> Res<u32>,
    ) -> Self {
        Arg {
            arg,
            display,
            value: ArgValue::U32(ArgT::<u32> {
                value: default,
                default,
                parser,
            }),
        }
    }

    pub const fn new_string(
        arg: &'static str,
        display: &'static str,
        default: Option<String>,
        parser: fn(&str) -> Res<String>,
    ) -> Self {
        Arg {
            arg,
            display,
            value: ArgValue::String(ArgT::<String> {
                // we can't duplicate the default value here because the function is const
                // the default value will be set in a call to reset_default
                value: None,
                default,
                parser,
            }),
        }
    }

    pub fn is_active(&self) -> bool {
        match &self.value {
            ArgValue::Bool(x) => x.is_set(),
            ArgValue::String(x) => x.is_set(),
            ArgValue::U32(x) => x.is_set(),
        }
    }

    pub fn unset(&mut self) -> () {
        match &mut self.value {
            ArgValue::Bool(x) => x.unset(),
            ArgValue::String(x) => x.unset(),
            ArgValue::U32(x) => x.unset(),
        }
    }

    pub fn set(&mut self, value: &str) -> Res<()> {
        match &mut self.value {
            ArgValue::Bool(x) => x.set(value),
            ArgValue::String(x) => x.set(value),
            ArgValue::U32(x) => x.set(value),
        }
    }

    pub fn expects_value(&self) -> bool {
        match &self.value {
            ArgValue::Bool(x) => x.expects_value(),
            ArgValue::String(x) => x.expects_value(),
            ArgValue::U32(x) => x.expects_value(),
        }
    }

    pub fn reset_default(&mut self) -> () {
        match &mut self.value {
            ArgValue::Bool(x) => x.reset_default(),
            ArgValue::String(x) => x.reset_default(),
            ArgValue::U32(x) => x.reset_default(),
        }
    }

    pub fn default_as_string(&self) -> Option<String> {
        match &self.value {
            ArgValue::Bool(x) => x.default_as_string(),
            ArgValue::String(x) => x.default_as_string(),
            ArgValue::U32(x) => x.default_as_string(),
        }
    }

    pub fn get_u32(&self) -> Option<u32> {
        match &self.value {
            ArgValue::Bool(_) => None,
            ArgValue::String(_) => None,
            ArgValue::U32(x) => x.value.clone(),
        }
    }

    pub fn value_as_string(&self) -> Option<String> {
        match &self.value {
            ArgValue::Bool(x) => x.value_as_string(),
            ArgValue::String(x) => x.value_as_string(),
            ArgValue::U32(x) => x.value_as_string(),
        }
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
    fn reset_default(&mut self) -> ();
    fn expects_value(&self) -> bool;
    fn default_as_string(&self) -> Option<String>;
    fn set(&mut self, value: &str) -> Res<()>;
    fn value_as_string(&self) -> Option<String>;
}

// trait ArgValue: ArgValueBase + core::fmt::Debug + DynClone {}

// dyn_clone::clone_trait_object!(ArgValue);

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

    fn reset_default(&mut self) -> () {}

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
}

// impl ArgValue for ArgBool {}

#[derive(Debug, Clone)]
struct ArgT<T> {
    value: Option<T>,
    default: Option<T>,
    parser: fn(&str) -> Res<T>,
}

impl<T> ArgValueBase for ArgT<T>
where
    T: Clone + std::fmt::Display,
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

    fn reset_default(&mut self) -> () {
        self.value = self.default.clone();
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
}

// impl<T> ArgValue for ArgT<T> where T: Clone + std::fmt::Debug + std::fmt::Display {}

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

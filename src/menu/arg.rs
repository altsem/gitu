use figment::value;

use crate::Res;

#[derive(Debug, Clone)]
pub(crate) enum ArgValue {
    Bool(bool),
    String(String),
    NumberOpt(Option<u32>),
}

#[derive(Debug, Clone)]
pub(crate) struct Arg {
    pub arg: &'static str,
    pub display: &'static str,
    pub value: ArgValue,
}

impl Arg {
    pub const fn new(arg: &'static str, display: &'static str, default: bool) -> Self {
        Arg {
            arg,
            display,
            value: ArgValue::Bool(default),
        }
    }

    pub const fn new_str(arg: &'static str, display: &'static str) -> Self {
        Arg {
            arg,
            display,
            value: ArgValue::String(String::new()),
        }
    }

    pub const fn new_int_opt(
        arg: &'static str,
        display: &'static str,
        default: Option<u32>,
    ) -> Self {
        Arg {
            arg,
            display,
            value: ArgValue::NumberOpt(default),
        }
    }

    pub fn is_active(&self) -> bool {
        match &self.value {
            ArgValue::Bool(state) => *state,
            ArgValue::String(state) => !state.is_empty(),
            ArgValue::NumberOpt(state) => state.is_some(),
        }
    }

    pub fn unset(&mut self) -> () {
        match &self.value {
            ArgValue::Bool(_) => self.value = ArgValue::Bool(false),
            ArgValue::String(_) => self.value = ArgValue::String(String::new()),
            ArgValue::NumberOpt(_) => self.value = ArgValue::NumberOpt(None),
        }
    }

    pub fn set(&mut self, value: &str) -> Res<()> {
        match &self.value {
            ArgValue::Bool(_) => {
                self.value = ArgValue::Bool(false);
                Ok(())
            }
            ArgValue::String(_) => {
                self.value = ArgValue::String(value.to_string());
                Ok(())
            }
            ArgValue::NumberOpt(_) => {
                let value = value.parse::<u32>()?;
                if value == 0 {
                    Err(String::from("Value must be a positive integer").into())
                } else {
                    self.value = ArgValue::NumberOpt(Some(value));
                    Ok(())
                }
            }
        }
    }

    pub fn get_u32(&self) -> Option<u32> {
        match &self.value {
            ArgValue::NumberOpt(state) => *state,
            _ => None,
        }
    }

    fn get_value_suffix(&self) -> String {
        match &self.value {
            ArgValue::String(state) if !state.is_empty() => format!("={}", state),
            ArgValue::NumberOpt(Some(state)) => format!("={}", state),
            _ => String::new(),
        }
    }

    pub fn get_cli_token(&self) -> String {
        format!("{}{}", self.arg, self.get_value_suffix())
    }
}

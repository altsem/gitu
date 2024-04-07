#[derive(Debug, Clone)]
pub(crate) struct Arg {
    pub arg: &'static str,
    pub display: &'static str,
    pub(crate) default: bool,
    pub(crate) state: bool,
}

impl Arg {
    pub const fn new(arg: &'static str, display: &'static str, default: bool) -> Self {
        Arg {
            arg,
            display,
            default,
            state: default,
        }
    }

    pub fn toggle(&mut self) {
        self.state = !self.state;
    }

    pub fn is_acive(&self) -> bool {
        self.state != self.default
    }
}

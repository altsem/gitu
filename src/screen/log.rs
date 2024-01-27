use super::ScreenData;
use crate::{git, items};

pub(crate) struct LogData {
    log: String,
}

impl LogData {
    pub(crate) fn capture(args: &[String]) -> Self {
        let log = git::log(&args.iter().map(String::as_str).collect::<Vec<_>>());
        Self { log }
    }
}

impl ScreenData for LogData {
    fn items<'a>(&'a self) -> Vec<items::Item> {
        items::create_log_items(&self.log).collect()
    }
}

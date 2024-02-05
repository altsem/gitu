use super::Screen;
use crate::{
    git,
    items::{self},
    util, Config, Res,
};

pub(crate) fn create(config: &Config, args: Vec<String>) -> Res<Screen> {
    let path_buf = config.dir.clone();
    Screen::new(Box::new(move || {
        let str_args = util::str_vec(&args);
        let log = git::log(&path_buf, &str_args)?;

        Ok(items::create_log_items(&log).collect())
    }))
}

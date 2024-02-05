use super::Screen;
use crate::{git, items, util, Config, Res};

pub(crate) fn create(config: &Config, args: Vec<String>) -> Res<Screen> {
    let dir = config.dir.clone();
    Screen::new(Box::new(move || {
        let str_args = util::str_vec(&args);
        let diff = git::diff(&dir, &str_args)?;

        Ok(items::create_diff_items(&diff, &0).collect())
    }))
}

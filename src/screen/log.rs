use super::Screen;
use crate::{git, items};

pub(crate) fn create(size: (u16, u16), args: Vec<String>) -> Screen {
    let args_clone = args.clone();

    Screen::new(
        size,
        Box::new(move || {
            items::create_log_items(git::log(
                &args_clone.iter().map(String::as_str).collect::<Vec<_>>(),
            ))
            .collect()
        }),
    )
}

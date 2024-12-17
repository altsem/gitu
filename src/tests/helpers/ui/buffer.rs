use ratatui::buffer::Buffer;
use std::{
    fmt::{Formatter, Result},
    hash::{DefaultHasher, Hash, Hasher},
};
use unicode_width::UnicodeWidthStr;

pub(crate) struct TestBuffer<'a>(pub &'a Buffer);

impl std::fmt::Debug for TestBuffer<'_> {
    // Stolen from ratatui-0.26.1/src/buffer/buffer.rs:394
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut last_style = None;
        let mut styles = vec![];
        for (y, line) in self
            .0
            .content
            .chunks(self.0.area.width as usize)
            .enumerate()
        {
            let mut overwritten = vec![];
            let mut skip: usize = 0;
            for (x, c) in line.iter().enumerate() {
                if skip == 0 {
                    f.write_str(c.symbol())?;
                } else {
                    overwritten.push((x, c.symbol()));
                }
                skip = std::cmp::max(skip, c.symbol().width()).saturating_sub(1);
                {
                    let style = (c.fg, c.bg, c.underline_color, c.modifier);
                    if last_style != Some(style) {
                        last_style = Some(style);
                        styles.push((x, y, c.fg, c.bg, c.underline_color, c.modifier));
                    }
                }
            }
            if !overwritten.is_empty() {
                f.write_fmt(format_args!(
                    "// hidden by multi-width symbols: {overwritten:?}"
                ))?;
            }
            f.write_str("|\n")?;
        }

        f.write_str("styles_hash: ")?;
        let mut hasher = DefaultHasher::new();
        styles.hash(&mut hasher);
        f.write_fmt(format_args!("{:x}", hasher.finish()))?;
        Ok(())
    }
}

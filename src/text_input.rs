use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    #[default]
    Pending,
    Aborted,
    Done,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct TextInput {
    pub(crate) value: String,
    /// Counted in chars, not bytes.
    position: usize,
    pub(crate) status: Status,
    pub(crate) focused: bool,
}

impl TextInput {
    /// Splits the value around the char the cursor is on, so it can be drawn
    /// as a cursor rather than the cursor taking up a column of its own. That
    /// char is empty with the cursor at the end of the value.
    pub(crate) fn split_at_cursor(&self) -> (&str, &str, &str) {
        let byte_at = |position| {
            self.value
                .char_indices()
                .nth(position)
                .map_or(self.value.len(), |(byte, _)| byte)
        };

        let start = byte_at(self.position);
        let end = byte_at(self.position + 1);

        (
            &self.value[..start],
            &self.value[start..end],
            &self.value[end..],
        )
    }

    pub(crate) fn handle_key_event(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }

        match (key.code, key.modifiers) {
            (KeyCode::Enter, _) => self.status = Status::Done,
            (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.status = Status::Aborted;
            }
            (KeyCode::Left, _) | (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                self.position = self.position.saturating_sub(1);
            }
            (KeyCode::Right, _) | (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.position = self.position.saturating_add(1).min(self.len());
            }
            (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => self.position = 0,
            (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.position = self.len();
            }
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                if self.position > 0 {
                    self.remove(self.position - 1);
                    self.position -= 1;
                }
            }
            (KeyCode::Delete, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                if self.position < self.len() {
                    self.remove(self.position);
                }
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                self.value = self.value.chars().take(self.position).collect();
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.value.clear();
                self.position = 0;
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.insert(c);
                self.position += 1;
            }
            _ => (),
        }
    }

    fn len(&self) -> usize {
        self.value.chars().count()
    }

    /// Editing goes via chars, as `String`'s own byte-indexed methods would
    /// split multi-byte ones.
    fn remove(&mut self, position: usize) {
        self.value = self
            .value
            .chars()
            .take(position)
            .chain(self.value.chars().skip(position + 1))
            .collect();
    }

    fn insert(&mut self, c: char) {
        if self.position == self.len() {
            self.value.push(c);
            return;
        }

        self.value = self
            .value
            .chars()
            .take(self.position)
            .chain(std::iter::once(c))
            .chain(self.value.chars().skip(self.position))
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(input: &mut TextInput, code: KeyCode) {
        input.handle_key_event(KeyEvent::new(code, KeyModifiers::empty()));
    }

    fn ctrl(input: &mut TextInput, c: char) {
        input.handle_key_event(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
    }

    fn typed(text: &str) -> TextInput {
        let mut input = TextInput::default();
        for c in text.chars() {
            press(&mut input, KeyCode::Char(c));
        }
        input
    }

    #[test]
    fn types_and_edits() {
        let mut input = typed("abc");
        assert_eq!(input.value, "abc");

        press(&mut input, KeyCode::Backspace);
        assert_eq!(input.value, "ab");

        press(&mut input, KeyCode::Left);
        press(&mut input, KeyCode::Char('x'));
        assert_eq!(input.value, "axb");

        press(&mut input, KeyCode::Delete);
        assert_eq!(input.value, "ax");
    }

    #[test]
    fn edits_multibyte_chars() {
        let mut input = typed("åäö");

        press(&mut input, KeyCode::Backspace);
        assert_eq!(input.value, "åä");

        press(&mut input, KeyCode::Home);
        press(&mut input, KeyCode::Delete);
        assert_eq!(input.value, "ä");

        press(&mut input, KeyCode::Char('ö'));
        assert_eq!(input.value, "öä");
    }

    #[test]
    fn moves_within_bounds() {
        let mut input = typed("ab");

        press(&mut input, KeyCode::Right);
        press(&mut input, KeyCode::Right);
        press(&mut input, KeyCode::Char('c'));
        assert_eq!(input.value, "abc");

        press(&mut input, KeyCode::Home);
        press(&mut input, KeyCode::Left);
        press(&mut input, KeyCode::Char('x'));
        assert_eq!(input.value, "xabc");
    }

    #[test]
    fn kills_and_truncates() {
        let mut input = typed("abcd");
        press(&mut input, KeyCode::Left);
        ctrl(&mut input, 'k');
        assert_eq!(input.value, "abc");

        ctrl(&mut input, 'u');
        assert_eq!(input.value, "");

        press(&mut input, KeyCode::Char('e'));
        assert_eq!(input.value, "e");
    }

    #[test]
    fn tracks_status() {
        let mut input = TextInput::default();
        assert_eq!(input.status, Status::Pending);

        press(&mut input, KeyCode::Enter);
        assert_eq!(input.status, Status::Done);

        let mut input = TextInput::default();
        press(&mut input, KeyCode::Esc);
        assert_eq!(input.status, Status::Aborted);

        let mut input = TextInput::default();
        ctrl(&mut input, 'c');
        assert_eq!(input.status, Status::Aborted);
    }
}

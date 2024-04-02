use crossterm::event::{KeyCode, KeyModifiers};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, none_of},
    combinator::{all_consuming, map, opt, value},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded},
    IResult,
};

// TODO Improve error messages

pub(crate) fn parse_keys(input: &str) -> IResult<&str, Vec<(KeyModifiers, KeyCode)>> {
    all_consuming(many0(parse_key))(input)
}

fn parse_key(input: &str) -> IResult<&str, (KeyModifiers, KeyCode)> {
    alt((parse_quoted, parse_char_key))(input)
}

fn parse_quoted(input: &str) -> IResult<&str, (KeyModifiers, KeyCode)> {
    delimited(char('<'), parse_modifiers_and_key, char('>'))(input)
}

fn parse_modifiers_and_key(input: &str) -> IResult<&str, (KeyModifiers, KeyCode)> {
    let (input, mods_vec) = separated_list0(tag("+"), parse_modifier)(input)?;
    let mods = mods_vec
        .into_iter()
        .reduce(KeyModifiers::union)
        .unwrap_or(KeyModifiers::NONE);

    preceded(opt(tag("+")), alt((parse_special_key, parse_char_key)))(input)
        .map(|(rem, (m, key))| (rem, (m.union(mods), key)))
}

fn parse_special_key(input: &str) -> IResult<&str, (KeyModifiers, KeyCode)> {
    alt((
        value(KeyCode::Backspace, tag("backspace")),
        value(KeyCode::Enter, tag("enter")),
        value(KeyCode::Left, tag("left")),
        value(KeyCode::Right, tag("right")),
        value(KeyCode::Up, tag("up")),
        value(KeyCode::Down, tag("down")),
        value(KeyCode::Home, tag("home")),
        value(KeyCode::End, tag("end")),
        value(KeyCode::PageUp, tag("pageup")),
        value(KeyCode::PageDown, tag("pagedown")),
        value(KeyCode::Tab, tag("tab")),
        value(KeyCode::BackTab, tag("backtab")),
        value(KeyCode::Delete, tag("delete")),
        value(KeyCode::Insert, tag("insert")),
        value(KeyCode::Esc, tag("esc")),
        value(KeyCode::CapsLock, tag("capslock")),
    ))(input)
    .map(|(rem, key)| (rem, (KeyModifiers::NONE, key)))
}

fn parse_modifier(input: &str) -> IResult<&str, KeyModifiers> {
    alt((
        value(KeyModifiers::SHIFT, tag("shift")),
        value(KeyModifiers::CONTROL, tag("ctrl")),
        value(KeyModifiers::ALT, tag("alt")),
        value(KeyModifiers::SUPER, tag("super")),
        value(KeyModifiers::HYPER, tag("hyper")),
        value(KeyModifiers::META, tag("meta")),
    ))(input)
}

fn parse_char_key(input: &str) -> IResult<&str, (KeyModifiers, KeyCode)> {
    none_of("<>")(input)?;
    map(anychar, |c| {
        let modifiers = if c.is_uppercase() {
            KeyModifiers::SHIFT
        } else {
            KeyModifiers::NONE
        };

        (modifiers, KeyCode::Char(c))
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use KeyCode::*;

    #[test]
    fn single_char() {
        assert_eq!(
            parse_keys("a"),
            Ok(("", vec![(KeyModifiers::NONE, Char('a'))]))
        );
    }

    #[test]
    fn upper_char() {
        assert_eq!(
            parse_keys("A"),
            Ok(("", vec![(KeyModifiers::SHIFT, Char('A'))]))
        );
    }

    #[test]
    fn special_key() {
        assert_eq!(
            parse_keys("<backspace>"),
            Ok(("", vec![(KeyModifiers::NONE, KeyCode::Backspace)]))
        );
    }

    #[test]
    fn modifier() {
        assert_eq!(
            parse_keys("<ctrl+j>"),
            Ok(("", vec![(KeyModifiers::CONTROL, KeyCode::Char('j'))]))
        );
    }

    #[test]
    fn multiple_modifiers() {
        assert_eq!(
            parse_keys("<shift+ctrl+alt+k>"),
            Ok((
                "",
                vec![(
                    KeyModifiers::SHIFT
                        .union(KeyModifiers::CONTROL)
                        .union(KeyModifiers::ALT),
                    KeyCode::Char('k')
                )]
            ))
        );
    }

    #[test]
    fn multiple() {
        assert_eq!(
            parse_keys("1<alt+end>A"),
            Ok((
                "",
                vec![
                    (KeyModifiers::NONE, Char('1')),
                    (KeyModifiers::ALT, End),
                    (KeyModifiers::SHIFT, Char('A')),
                ]
            ))
        );
    }
}

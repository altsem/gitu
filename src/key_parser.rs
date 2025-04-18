use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, none_of},
    combinator::{all_consuming, map, opt, value},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded},
    IResult,
};
use termwiz::input::{KeyCode, Modifiers};

// TODO Improve error messages

pub(crate) fn parse_keys(input: &str) -> IResult<&str, Vec<(Modifiers, KeyCode)>> {
    all_consuming(many0(parse_key))(input)
}

fn parse_key(input: &str) -> IResult<&str, (Modifiers, KeyCode)> {
    alt((parse_quoted, parse_char_key))(input)
}

fn parse_quoted(input: &str) -> IResult<&str, (Modifiers, KeyCode)> {
    delimited(char('<'), parse_modifiers_and_key, char('>'))(input)
}

fn parse_modifiers_and_key(input: &str) -> IResult<&str, (Modifiers, KeyCode)> {
    let (input, mods_vec) = separated_list0(tag("+"), parse_modifier)(input)?;
    let mods = mods_vec
        .into_iter()
        .reduce(Modifiers::union)
        .unwrap_or(Modifiers::NONE);

    preceded(opt(tag("+")), alt((parse_special_key, parse_char_key)))(input)
        .map(|(rem, (m, key))| (rem, (m.union(mods), key)))
}

fn parse_special_key(input: &str) -> IResult<&str, (Modifiers, KeyCode)> {
    alt((
        value(KeyCode::Backspace, tag("backspace")),
        value(KeyCode::Enter, tag("enter")),
        value(KeyCode::LeftArrow, tag("left")),
        value(KeyCode::RightArrow, tag("right")),
        value(KeyCode::UpArrow, tag("up")),
        value(KeyCode::DownArrow, tag("down")),
        value(KeyCode::Home, tag("home")),
        value(KeyCode::End, tag("end")),
        value(KeyCode::PageUp, tag("pageup")),
        value(KeyCode::PageDown, tag("pagedown")),
        value(KeyCode::Tab, tag("tab")),
        // FIXME Drop this and mention in changelog
        // value(KeyCode::BackTab, tag("backtab")),
        value(KeyCode::Delete, tag("delete")),
        value(KeyCode::Insert, tag("insert")),
        value(KeyCode::Escape, tag("esc")),
        value(KeyCode::CapsLock, tag("capslock")),
    ))(input)
    .map(|(rem, key)| (rem, (Modifiers::NONE, key)))
}

fn parse_modifier(input: &str) -> IResult<&str, Modifiers> {
    alt((
        value(Modifiers::SHIFT, tag("shift")),
        value(Modifiers::CTRL, tag("ctrl")),
        value(Modifiers::ALT, tag("alt")),
        value(Modifiers::SUPER, tag("super")),
        // FIXME Drop these and mention in changelog
        // value(Modifiers::HYPER, tag("hyper")),
        // value(Modifiers::META, tag("meta")),
    ))(input)
}

fn parse_char_key(input: &str) -> IResult<&str, (Modifiers, KeyCode)> {
    none_of("<>")(input)?;
    map(anychar, |c| (Modifiers::NONE, KeyCode::Char(c)))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use termwiz::input::Modifiers;
    use KeyCode::*;

    #[test]
    fn single_char() {
        assert_eq!(
            parse_keys("a"),
            Ok(("", vec![(Modifiers::NONE, Char('a'))]))
        );
    }

    #[test]
    fn upper_char() {
        assert_eq!(
            parse_keys("A"),
            Ok(("", vec![(Modifiers::NONE, Char('A'))]))
        );
    }

    #[test]
    fn special_key() {
        assert_eq!(
            parse_keys("<backspace>"),
            Ok(("", vec![(Modifiers::NONE, KeyCode::Backspace)]))
        );
    }

    #[test]
    fn modifier() {
        assert_eq!(
            parse_keys("<ctrl+j>"),
            Ok(("", vec![(Modifiers::CTRL, KeyCode::Char('j'))]))
        );
    }

    #[test]
    fn multiple_modifiers() {
        assert_eq!(
            parse_keys("<shift+ctrl+alt+k>"),
            Ok((
                "",
                vec![(
                    Modifiers::SHIFT
                        .union(Modifiers::CTRL)
                        .union(Modifiers::ALT),
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
                    (Modifiers::NONE, Char('1')),
                    (Modifiers::ALT, End),
                    (Modifiers::NONE, Char('A')),
                ]
            ))
        );
    }
}

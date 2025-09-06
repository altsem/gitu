use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char},
    combinator::{all_consuming, map},
    error::{Error, ErrorKind},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, tuple},
    Err, IResult,
};
use termwiz::input::{KeyCode, Modifiers};

pub(crate) fn parse_keys(input: &str) -> IResult<&str, Vec<(Modifiers, KeyCode)>> {
    map(parse_key_string, |keys| {
        keys.into_iter().map(|key| (Modifiers::NONE, key)).collect()
    })(input)
}

/// Parse a string into [`Modifiers`] and [`KeyCode`]s. This function is
/// intended to be used only when parsing the key bindings in a config file.
pub(crate) fn parse_config_keys(input: &str) -> IResult<&str, Vec<(Modifiers, KeyCode)>> {
    map(parse_key_combo, |(modifiers, keys)| {
        keys.into_iter().map(|key| (modifiers, key)).collect()
    })(input)
}

/// Parse a string of keys that lack '+' delimiters into [`Modifiers`] and
/// [`KeyCode`]s. This function is intended to be used only in tests.
#[cfg(test)]
pub(crate) fn parse_test_keys(input: &str) -> IResult<&str, Vec<(Modifiers, KeyCode)>> {
    use nom::{combinator::opt, multi::fold_many0};

    all_consuming(fold_many0(
        alt((
            map(parse_prefixed, |keys| {
                keys.into_iter()
                    .map(|key| (Modifiers::NONE, key))
                    .collect::<Vec<_>>()
            }),
            delimited(
                char('<'),
                map(
                    tuple((opt(parse_modifiers), opt(char('+')), parse_normal_key)),
                    |(modifiers, _, key)| vec![(modifiers.unwrap_or_default(), key)],
                ),
                char('>'),
            ),
            map(parse_char_key, |key| vec![(Modifiers::NONE, key)]),
        )),
        Vec::new,
        |mut acc, items| {
            acc.extend(items);
            acc
        },
    ))(input.trim())
}

/// Parse a string of keys *that aren't* delimited by '+' into a key
/// combination. There can be multiple "normal" keys and modifiers (represented
/// as a single `Modifiers`).
fn parse_key_string(input: &str) -> IResult<&str, Vec<KeyCode>> {
    all_consuming(many1(parse_normal_key))(input)
}

/// Parse a string of keys delimited by '+' into a key combination. There can be
/// multiple "normal" keys and modifiers (represented as a single `Modifiers`).
fn parse_key_combo(input: &str) -> IResult<&str, (Modifiers, Vec<KeyCode>)> {
    let input = input.trim();
    if input.trim().is_empty() {
        return Err(Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    all_consuming(alt((
        map(parse_prefixed, |keys| (Modifiers::NONE, keys)),
        parse_multiple_keys,
    )))(input)
}

fn parse_prefixed(input: &str) -> IResult<&str, Vec<KeyCode>> {
    map(preceded(char('-'), parse_char_key), |key| {
        vec![KeyCode::Char('-'), key]
    })(input)
}

fn parse_multiple_keys(input: &str) -> IResult<&str, (Modifiers, Vec<KeyCode>)> {
    alt((
        map(
            tuple((
                parse_modifiers,
                char('+'),
                separated_list1(char('+'), parse_normal_key),
            )),
            |(modifiers, _, keys)| (modifiers, keys),
        ),
        map(separated_list1(char('+'), parse_normal_key), |keys| {
            (Modifiers::NONE, keys)
        }),
    ))(input)
}

fn parse_modifiers(input: &str) -> IResult<&str, Modifiers> {
    map(separated_list1(char('+'), parse_modifier), |modifiers| {
        modifiers
            .into_iter()
            .fold(Modifiers::NONE, |acc, m| acc | m)
    })(input)
}

fn parse_modifier(input: &str) -> IResult<&str, Modifiers> {
    alt((
        map(tag("shift"), |_| Modifiers::SHIFT),
        map(tag("ctrl"), |_| Modifiers::CTRL),
        map(tag("alt"), |_| Modifiers::ALT),
        map(tag("super"), |_| Modifiers::SUPER),
        // FIXME Drop these and mention in changelog
        // map(tag("hyper"), |_| Modifiers::HYPER),
        // map(tag("meta"), |_| Modifiers::META),
    ))(input)
}

fn parse_normal_key(input: &str) -> IResult<&str, KeyCode> {
    alt((
        delimited(char('<'), parse_special_key, char('>')),
        parse_special_key,
        parse_char_key,
    ))(input)
}

fn parse_char_key(input: &str) -> IResult<&str, KeyCode> {
    map(anychar, KeyCode::Char)(input)
}

fn parse_special_key(input: &str) -> IResult<&str, KeyCode> {
    alt((
        alt((
            map(tag("backspace"), |_| KeyCode::Backspace),
            map(tag("enter"), |_| KeyCode::Enter),
            map(tag("left"), |_| KeyCode::LeftArrow),
            map(tag("right"), |_| KeyCode::RightArrow),
            map(tag("up"), |_| KeyCode::UpArrow),
            map(tag("down"), |_| KeyCode::DownArrow),
            map(tag("home"), |_| KeyCode::Home),
            map(tag("end"), |_| KeyCode::End),
            map(tag("pageup"), |_| KeyCode::PageUp),
            map(tag("pagedown"), |_| KeyCode::PageDown),
            map(tag("tab"), |_| KeyCode::Tab),
            map(tag("delete"), |_| KeyCode::Delete),
            map(tag("insert"), |_| KeyCode::Insert),
            map(tag("esc"), |_| KeyCode::Escape),
            map(tag("capslock"), |_| KeyCode::CapsLock),
        )),
        // TODO: `alt` only allows upto 21 entries. If possible, combine later.
        alt((
            map(tag("f1"), |_| KeyCode::Function(1)),
            map(tag("f2"), |_| KeyCode::Function(2)),
            map(tag("f3"), |_| KeyCode::Function(3)),
            map(tag("f4"), |_| KeyCode::Function(4)),
            map(tag("f5"), |_| KeyCode::Function(5)),
            map(tag("f6"), |_| KeyCode::Function(6)),
            map(tag("f7"), |_| KeyCode::Function(7)),
            map(tag("f8"), |_| KeyCode::Function(8)),
            map(tag("f9"), |_| KeyCode::Function(9)),
            map(tag("f10"), |_| KeyCode::Function(10)),
            map(tag("f11"), |_| KeyCode::Function(11)),
            map(tag("f12"), |_| KeyCode::Function(12)),
        )),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use termwiz::input::Modifiers;
    use KeyCode::*;

    #[test]
    fn single_char() {
        assert_eq!(
            parse_key_combo("a"),
            Ok(("", (Modifiers::NONE, vec![Char('a')])))
        );
    }

    #[test]
    fn upper_char() {
        assert_eq!(
            parse_key_combo("A"),
            Ok(("", (Modifiers::NONE, vec![Char('A')])))
        );
    }

    #[test]
    fn special_key() {
        assert_eq!(
            parse_key_combo("backspace"),
            Ok(("", (Modifiers::NONE, vec![KeyCode::Backspace])))
        );
        assert_eq!(
            parse_key_combo("enter"),
            Ok(("", (Modifiers::NONE, vec![KeyCode::Enter])))
        );
    }

    #[test]
    fn modifier() {
        assert_eq!(
            parse_key_combo("ctrl+j"),
            Ok(("", (Modifiers::CTRL, vec![KeyCode::Char('j')])))
        );

        // "ctrla" is invalid
        assert!(parse_key_combo("ctrla").is_err());
    }

    #[test]
    fn multiple_modifiers() {
        assert_eq!(
            parse_key_combo("shift+ctrl+alt+k"),
            Ok((
                "",
                (
                    Modifiers::SHIFT
                        .union(Modifiers::CTRL)
                        .union(Modifiers::ALT),
                    vec![KeyCode::Char('k')]
                )
            ))
        );
    }

    #[test]
    fn multiple() {
        assert_eq!(
            parse_key_combo("alt+end"),
            Ok(("", (Modifiers::ALT, vec![End])))
        );
    }

    #[test]
    fn dash_a() {
        assert_eq!(
            parse_key_combo("-a"),
            Ok((
                "",
                (
                    Modifiers::NONE,
                    vec![KeyCode::Char('-'), KeyCode::Char('a')]
                )
            ))
        );
    }

    #[test]
    fn test_parser() {
        assert_eq!(
            parse_test_keys("1<alt+end>A"),
            Ok((
                "",
                vec![
                    (Modifiers::NONE, KeyCode::Char('1')),
                    (Modifiers::ALT, End),
                    (Modifiers::NONE, KeyCode::Char('A'))
                ]
            ))
        );
    }
}

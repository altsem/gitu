use crossterm::event::{KeyCode, KeyModifiers};
use nom::{
    Err, IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char},
    combinator::{all_consuming, map},
    error::{Error, ErrorKind},
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, tuple},
};

pub(crate) fn parse_keys(input: &str) -> IResult<&str, Vec<(KeyModifiers, KeyCode)>> {
    map(parse_key_string, |keys| {
        keys.into_iter()
            .map(|key| (KeyModifiers::NONE, key))
            .collect()
    })(input)
}

/// Parse a string into [`Modifiers`] and [`KeyCode`]s. This function is
/// intended to be used only when parsing the key bindings in a config file.
pub(crate) fn parse_config_keys(input: &str) -> IResult<&str, Vec<(KeyModifiers, KeyCode)>> {
    map(parse_key_combo, |(modifiers, keys)| {
        keys.into_iter().map(|key| (modifiers, key)).collect()
    })(input)
}

/// Parse a string of keys that lack '+' delimiters into [`KeyModifiers`] and
/// [`KeyCode`]s. This function is intended to be used only in tests.
#[cfg(test)]
pub(crate) fn parse_test_keys(input: &str) -> IResult<&str, Vec<(KeyModifiers, KeyCode)>> {
    use nom::{combinator::opt, multi::fold_many0};

    all_consuming(fold_many0(
        alt((
            map(parse_prefixed, |keys| {
                keys.into_iter()
                    .map(|key| (KeyModifiers::NONE, key))
                    .collect::<Vec<_>>()
            }),
            delimited(
                char('<'),
                map(
                    tuple((opt(parse_modifiers), opt(char('+')), parse_normal_key)),
                    |(modifiers, _, key)| vec![(modifiers.unwrap_or(KeyModifiers::NONE), key)],
                ),
                char('>'),
            ),
            map(parse_char_key, |key| vec![(KeyModifiers::NONE, key)]),
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
/// as a single `KeyModifiers`).
fn parse_key_string(input: &str) -> IResult<&str, Vec<KeyCode>> {
    all_consuming(many1(parse_normal_key))(input)
}

/// Parse a string of keys delimited by '+' into a key combination. There can be
/// multiple "normal" keys and modifiers (represented as a single `Modifiers`).
fn parse_key_combo(input: &str) -> IResult<&str, (KeyModifiers, Vec<KeyCode>)> {
    let input = input.trim();
    if input.trim().is_empty() {
        return Err(Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    all_consuming(alt((
        map(parse_prefixed, |keys| (KeyModifiers::NONE, keys)),
        parse_multiple_keys,
    )))(input)
}

fn parse_prefixed(input: &str) -> IResult<&str, Vec<KeyCode>> {
    map(preceded(char('-'), parse_char_key), |key| {
        vec![KeyCode::Char('-'), key]
    })(input)
}

fn parse_multiple_keys(input: &str) -> IResult<&str, (KeyModifiers, Vec<KeyCode>)> {
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
            (KeyModifiers::NONE, keys)
        }),
    ))(input)
}

fn parse_modifiers(input: &str) -> IResult<&str, KeyModifiers> {
    map(separated_list1(char('+'), parse_modifier), |modifiers| {
        modifiers
            .into_iter()
            .fold(KeyModifiers::NONE, |acc, m| acc | m)
    })(input)
}

fn parse_modifier(input: &str) -> IResult<&str, KeyModifiers> {
    alt((
        map(tag("shift"), |_| KeyModifiers::SHIFT),
        map(tag("ctrl"), |_| KeyModifiers::CONTROL),
        map(tag("alt"), |_| KeyModifiers::ALT),
        map(tag("super"), |_| KeyModifiers::SUPER),
        map(tag("hyper"), |_| KeyModifiers::HYPER),
        map(tag("meta"), |_| KeyModifiers::META),
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
            map(tag("left"), |_| KeyCode::Left),
            map(tag("right"), |_| KeyCode::Right),
            map(tag("up"), |_| KeyCode::Up),
            map(tag("down"), |_| KeyCode::Down),
            map(tag("home"), |_| KeyCode::Home),
            map(tag("end"), |_| KeyCode::End),
            map(tag("pageup"), |_| KeyCode::PageUp),
            map(tag("pagedown"), |_| KeyCode::PageDown),
            map(tag("tab"), |_| KeyCode::Tab),
            map(tag("delete"), |_| KeyCode::Delete),
            map(tag("insert"), |_| KeyCode::Insert),
            map(tag("esc"), |_| KeyCode::Esc),
            map(tag("capslock"), |_| KeyCode::CapsLock),
        )),
        // TODO: `alt` only allows upto 21 entries. If possible, combine later.
        alt((
            map(tag("f1"), |_| KeyCode::F(1)),
            map(tag("f2"), |_| KeyCode::F(2)),
            map(tag("f3"), |_| KeyCode::F(3)),
            map(tag("f4"), |_| KeyCode::F(4)),
            map(tag("f5"), |_| KeyCode::F(5)),
            map(tag("f6"), |_| KeyCode::F(6)),
            map(tag("f7"), |_| KeyCode::F(7)),
            map(tag("f8"), |_| KeyCode::F(8)),
            map(tag("f9"), |_| KeyCode::F(9)),
            map(tag("f10"), |_| KeyCode::F(10)),
            map(tag("f11"), |_| KeyCode::F(11)),
            map(tag("f12"), |_| KeyCode::F(12)),
        )),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use KeyCode::*;

    #[test]
    fn single_char() {
        assert_eq!(
            parse_key_combo("a"),
            Ok(("", (KeyModifiers::NONE, vec![Char('a')])))
        );
    }

    #[test]
    fn upper_char() {
        assert_eq!(
            parse_key_combo("A"),
            Ok(("", (KeyModifiers::NONE, vec![Char('A')])))
        );
    }

    #[test]
    fn special_key() {
        assert_eq!(
            parse_key_combo("backspace"),
            Ok(("", (KeyModifiers::NONE, vec![KeyCode::Backspace])))
        );
        assert_eq!(
            parse_key_combo("enter"),
            Ok(("", (KeyModifiers::NONE, vec![KeyCode::Enter])))
        );
    }

    #[test]
    fn modifier() {
        assert_eq!(
            parse_key_combo("ctrl+j"),
            Ok(("", (KeyModifiers::CONTROL, vec![KeyCode::Char('j')])))
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
                    KeyModifiers::SHIFT
                        .union(KeyModifiers::CONTROL)
                        .union(KeyModifiers::ALT),
                    vec![KeyCode::Char('k')]
                )
            ))
        );
    }

    #[test]
    fn multiple() {
        assert_eq!(
            parse_key_combo("alt+end"),
            Ok(("", (KeyModifiers::ALT, vec![End])))
        );
    }

    #[test]
    fn dash_a() {
        assert_eq!(
            parse_key_combo("-a"),
            Ok((
                "",
                (
                    KeyModifiers::NONE,
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
                    (KeyModifiers::NONE, KeyCode::Char('1')),
                    (KeyModifiers::ALT, End),
                    (KeyModifiers::NONE, KeyCode::Char('A'))
                ]
            ))
        );
    }
}

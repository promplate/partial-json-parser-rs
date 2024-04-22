use nom::{
    bytes::complete::{escaped, take_while, take_while1},
    character::complete::{char, one_of},
    sequence::{preceded, terminated},
};

use super::{ErrCast, ErrRes, JsonType, ParseRes};

fn is_space(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\r' || c == '\n'
}

pub fn sp(i: &str) -> ParseRes<&str> {
    take_while(is_space)(i)
}

fn is_string_character(c: char) -> bool {
    //FIXME: should validate unicode character
    c != '"' && c != '\\'
}

pub(super) fn parse_string(i: &str) -> ParseRes<&str> {
    let res: Result<(&str, &str), nom::Err<ErrRes<&str>>> = preceded(
        char('\"'),
        terminated(
            escaped(take_while1(is_string_character), '\\', one_of("\"bfnrt\\")),
            char('\"'),
        ),
    )(i);
    // debug_println!("string: {:#?}", res);
    res.cast(i, "\"", JsonType::Str, false)
}

pub(super) fn parse_key(i: &str) -> ParseRes<&str> {
    let res: Result<(&str, &str), nom::Err<ErrRes<&str>>> = preceded(
        char('\"'),
        terminated(
            escaped(take_while1(is_string_character), '\\', one_of("\"bfnrt\\")),
            char('\"'),
        ),
    )(i);
    res.cast(i, "", JsonType::Str, true)
}

#[cfg(test)]
mod test_string {
    use crate::partial_parser::ErrType;

    use super::*;

    #[test]
    fn test_string_complete_without_escaped() {
        let input = r#""laljfwej""#;
        let output = parse_string(input);
        // if let Ok((s1, s2)) = output {
        //     debug_println!("rem: {}, pattern: {}", s1, s2);
        // } else {
        //     debug_println!("Error!");
        // }
        assert_eq!(Ok(("", "laljfwej")), output);
    }

    #[test]
    fn test_string_complete_with_escaped() {
        let input = r#""laljfw\n\fe\"j""#;
        let output = parse_string(input);
        assert_eq!(Ok(("", r#"laljfw\n\fe\"j"#)), output);
    }

    #[test]
    fn test_string_invalid() {
        let input = r#""laljfw\q""#;
        let output = parse_string(input);
        match output {
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                assert_eq!(ErrType::Failure, err.err_type);
                assert_eq!(Some(input), err.err_str);
            }
            _ => panic!("Output is ok or completion!"),
        }
    }

    #[test]
    fn test_string_incomplete() {
        let input = r#""laljfw\""#;
        let output = parse_string(input);
        match output {
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                assert_eq!(
                    ErrType::Completion {
                        delete: false,
                        completion: r#"""#.to_string(),
                        json_type: JsonType::Str
                    },
                    err.err_type
                );
                assert_eq!(Some(input), err.err_str);
            }
            _ => panic!("Output is ok or completion!"),
        }
    }
}

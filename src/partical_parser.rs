use std::collections::HashMap;

use nom::bytes::complete::{escaped, tag, take, take_till, take_while, take_while1};
use nom::character::complete::{char, one_of};
use nom::combinator::{complete, cut, map, map_res};
use nom::error::{FromExternalError, ParseError, VerboseError};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::{IResult, Offset};

use crate::debug_println;

#[derive(Debug, PartialEq)]
pub enum JsonValue<'a> {
    Str(&'a str),
    Boolean(bool),
    Num(f64),
    Array(Vec<JsonValue<'a>>),
    Object(HashMap<&'a str, JsonValue<'a>>),
}

#[derive(Debug, PartialEq, Eq, Default)]
enum ErrType {
    #[default]
    Failure,
    Completion {
        completion: String,
        last_completion: Option<String>,
    },
}

#[derive(Debug, PartialEq)]
struct ErrRes<I> {
    pub base_err: VerboseError<I>,
    pub err_type: ErrType,
    pub idx: usize,
    pub ex_idx: usize,
}

impl<I> ErrRes<I> {
    // 对于不同的类型，需要不同的completion设置，所以
    fn simple_cast(base_err: VerboseError<I>) -> Self {
        Self {
            base_err,
            err_type: ErrType::default(),
            idx: usize::default(),
            ex_idx: usize::default(),
        }
    }
}

impl<I> ParseError<I> for ErrRes<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        ErrRes::simple_cast(VerboseError::from_error_kind(input, kind))
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.base_err = VerboseError::append(input, kind, other.base_err);
        other
    }
}

fn is_space(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\r' || c == '\n'
}

fn sp(i: &str) -> IResult<&str, &str> {
    take_while(is_space)(i)
}

pub fn is_string_character(c: char) -> bool {
    //FIXME: should validate unicode character
    c != '"' && c != '\\'
}

fn parse_string(i: &str) -> IResult<&str, &str, ErrRes<&str>> {
    let mut res: Result<(&str, &str), nom::Err<ErrRes<&str>>> = preceded(
        char('\"'),
        terminated(
            escaped(take_while1(is_string_character), '\\', one_of("\"bfnrt\\")),
            char('\"'),
        ),
    )(i);
    if let Err(ref mut err) = res {
        match err {
            nom::Err::Error(err) | nom::Err::Failure(err) => {
                let (rem, _) = err.base_err.errors.split_first().unwrap();
                let err_idx = i.offset(rem.0);
                debug_println!("rem: {}", rem.0);
                let completion = String::from("\"");
                err.err_type = if rem.0.is_empty() {
                    ErrType::Completion {
                        completion: String::default(),
                        last_completion: Some(completion),
                    }
                } else {
                    ErrType::Failure
                };
                
                err.ex_idx = 0;
                err.idx = err_idx;
            }
            _ => panic!("should not reach here"),
        }
    }
    res
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_println;


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
                assert_eq!(8, err.idx);
                assert_eq!(0, err.ex_idx);
            },
            _ => panic!("Output is ok or completion!")
        }
    }
}

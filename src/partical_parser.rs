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
        last_completion: String,
    },
}

trait ErrCast<'a> {
    fn cast(self, cur_str: &'a str, last_completion: String) -> Self;
}

impl <'a> ErrCast<'a> for IResult<&str, &str, ErrRes<'a, &str>> {
    fn cast(mut self, cur_str: &'a str, last_completion: String) -> Self {
        if let Err(ref mut err) = self {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    if err.err_str.is_none() {
                        // 如果是第一次发生错误，需要将err_str记录下来
                        let (rem, _) = err.base_err.errors.split_first().unwrap();
                        debug_println!("rem: {}", rem.0);
                        err.err_type = if rem.0.is_empty() {
                            ErrType::Completion {
                                completion: String::default(),
                                // 所有的completion都在返回上一级后提交（可能会丢弃）
                                last_completion,
                            }
                        } else {
                            ErrType::Failure
                        };
                        err.err_str = Some(cur_str)
                    }
                    // 不是第一次发生错误，不做处理直接将错误向后传递即可
                }
                _ => panic!("should not reach here"),
            }
        }
        self
    }
}

#[derive(Debug, PartialEq)]
struct ErrRes<'a, I> {
    pub base_err: VerboseError<I>,
    pub err_type: ErrType,
    pub err_str: Option<&'a str>,
}

impl<'a, I> ErrRes<'a, I> {
    // 对于不同的类型，需要不同的completion设置，所以
    fn simple_cast(base_err: VerboseError<I>) -> Self {
        Self {
            base_err,
            err_type: ErrType::default(),
            err_str: None,
        }
    }
}

impl<'a, I> ParseError<I> for ErrRes<'a, I> {
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
    let res: Result<(&str, &str), nom::Err<ErrRes<&str>>> = preceded(
        char('\"'),
        terminated(
            escaped(take_while1(is_string_character), '\\', one_of("\"bfnrt\\")),
            char('\"'),
        ),
    )(i);
    res.cast(i, String::from("\""))
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
                assert_eq!(Some(input), err.err_str);
            },
            _ => panic!("Output is ok or completion!")
        }
    }

    #[test]
    fn test_string_incomplete() {
        let input = r#""laljfw\""#;
        let output = parse_string(input);
        match output {
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                assert_eq!(ErrType::Completion { completion: String::default(), last_completion: r#"""#.to_string() }, err.err_type);
                assert_eq!(Some(input), err.err_str);
            },
            _ => panic!("Output is ok or completion!")
        }
    }
}

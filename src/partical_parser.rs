use std::collections::HashMap;

use nom::bytes::complete::{escaped, tag, take, take_till, take_while, take_while1};
use nom::character::complete::{char, one_of};
use nom::combinator::{complete, cut, map, map_res};
use nom::error::{FromExternalError, ParseError, VerboseError};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::{IResult, Offset};

use crate::debug_println;

#[derive(Debug, PartialEq, Eq, Default)]
pub enum JsonType {
    #[default]
    Str,
    Boolean,
    Num,
    KeyVal,
    Array,
    Object,
}

#[derive(Debug, PartialEq, Eq, Default)]
enum ErrType {
    #[default]
    Failure,
    Completion {
        json_type: JsonType,
        completion: String,
        last_completion: String,
    },
}

type ParseRes<'a, O> = IResult<&'a str, O, ErrRes<'a, &'a str>>;

trait ErrCast<'a> {
    fn cast(self, cur_str: &'a str, last_completion: String, json_type: JsonType) -> Self;
}

impl<'a, O> ErrCast<'a> for ParseRes<'a, O> {
    fn cast(mut self, cur_str: &'a str, last_completion: String, json_type: JsonType) -> Self {
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
                                json_type,
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

fn sp(i: &str) -> ParseRes<&str> {
    take_while(is_space)(i)
}

pub fn is_string_character(c: char) -> bool {
    //FIXME: should validate unicode character
    c != '"' && c != '\\'
}

fn parse_string(i: &str) -> ParseRes<&str> {
    let res: Result<(&str, &str), nom::Err<ErrRes<&str>>> = preceded(
        char('\"'),
        terminated(
            escaped(take_while1(is_string_character), '\\', one_of("\"bfnrt\\")),
            char('\"'),
        ),
    )(i);
    res.cast(i, String::from("\""), JsonType::Str)
}

fn tryd(i: &str) -> ParseRes<&str> {
    let res = terminated(parse_string, char('\"'))(i);
    res.cast(i, String::from("ds"), JsonType::Str)
}

fn parse_key_value(i: &str) -> ParseRes<(&str, &str)> {
    // 临时用parse_string顶替一下，测试
    let res: ParseRes<(&str, &str)> = separated_pair(
        preceded(sp, parse_string),
        cut(preceded(sp, char(':'))),
        parse_string,
    )(i);
    res.cast(i, "".to_string(), JsonType::KeyVal)
}

fn parse_boolean(i: &str) -> ParseRes<&str> {
    todo!()
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
                        completion: String::default(),
                        last_completion: r#"""#.to_string(),
                        json_type: JsonType::Str
                    },
                    err.err_type
                );
                assert_eq!(Some(input), err.err_str);
            }
            _ => panic!("Output is ok or completion!"),
        }
    }

    macro_rules! quick_test_failed {
        ($input: expr, $func: ident, $($eq_left: expr => $field: ident), +) => {{
            let output = $func($input);
            match output {
                Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                    $(
                        assert_eq!($eq_left, err.$field);
                    )+
                }
                _ => panic!("Output is ok or completion!"),
            }
        }}; 
    }

    #[test]
    fn test_keyval_incomplete() {
        quick_test_failed!(r#""laljfw""#, parse_key_value, 
            ErrType::Completion {
                completion: String::default(),
                last_completion: r#""#.to_string(),
                json_type: JsonType::KeyVal
            } => err_type,
            Some(r#""laljfw""#) => err_str
        )
    }

    #[test]
    fn test_keyval_invalid() {
        quick_test_failed!(r#""laljfw" ("#, parse_key_value, 
            ErrType::Failure => err_type,
            Some(r#""laljfw" ("#) => err_str
        )
    }
}

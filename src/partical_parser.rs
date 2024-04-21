use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::{escaped, tag, take, take_till, take_while, take_while1};
use nom::character::complete::{char, digit1, one_of};
use nom::combinator::{cut, opt, recognize};
use nom::error::{FromExternalError, ParseError, VerboseError};
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::{IResult, Offset};

use crate::{debug_println, utils};

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub enum JsonType {
    #[default]
    Str,
    Spec,
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
        delete: bool,
        json_type: JsonType,
        completion: String,
    },
}

type ParseRes<'a, O> = IResult<&'a str, O, ErrRes<'a, &'a str>>;

trait ErrCast<'a> {
    fn cast(
        self,
        cur_str: &'a str,
        cur_completion: &str,
        json_type: JsonType,
        delete: bool,
    ) -> Self;
    fn func_cast<F>(self, func: F, cur_completion: &str, delete: bool) -> Self
    where
        F: Fn(&mut ErrRes<'a, &str>);
    fn func_incomplete_cast<F>(self, func: F, cur_completion: &str, delete: bool) -> Self
    where
        F: Fn(&mut ErrRes<'a, &str>);
    fn incomplete_cast(
        self,
        cur_str: &'a str,
        cur_completion: &str,
        json_type: JsonType,
        delete: bool,
    ) -> Self;
    fn is_invalid(&self) -> bool;
    fn is_incomplete(&self) -> bool;
    fn is_delete(&self) -> bool;
}

impl<'a, O> ErrCast<'a> for ParseRes<'a, O> {
    fn cast(
        self,
        cur_str: &'a str,
        cur_completion: &str,
        json_type: JsonType,
        delete: bool,
    ) -> Self {
        self.func_cast(
            |err| {
                let (rem, _) = err.base_err.errors.split_first().unwrap();
                debug_println!("rem: {}", rem.0);
                let completion = if !delete {
                    cur_completion.to_string()
                } else {
                    String::new()
                };
                err.err_type = if rem.0.is_empty() {
                    ErrType::Completion {
                        delete,
                        completion,
                        json_type,
                    }
                } else {
                    ErrType::Failure
                };
                err.err_str = Some(cur_str)
            },
            cur_completion,
            delete,
        )
    }

    fn is_invalid(&self) -> bool {
        let mut invalid = false;
        if let Err(ref err) = self {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    if err.err_str.is_none() {
                        let (rem, _) = err.base_err.errors.split_first().unwrap();
                        invalid = !rem.0.is_empty();
                    } else {
                        invalid = err.err_type == ErrType::Failure;
                    }
                }
                _ => panic!("should not reach here: Incomplete Arm"),
            }
        }
        invalid
    }

    // 这里其实是有点问题的，对于非一个字符一个字符的匹配，它得到的结论是错误的
    // 比如说对于tag来说
    fn is_incomplete(&self) -> bool {
        self.is_err() && !self.is_invalid()
    }

    fn func_cast<F>(mut self, func: F, cur_completion: &str, delete: bool) -> Self
    where
        F: Fn(&mut ErrRes<'a, &str>),
    {
        if let Err(ref mut err) = self {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    if err.err_str.is_none() {
                        func(err);
                    } else {
                        // 不是第一次发生错误，需要对completion做补足
                        if let ErrType::Completion {
                            ref mut completion,
                            delete: ref mut delete_,
                            ..
                        } = err.err_type
                        {
                            *delete_ = delete;
                            if !delete {
                                completion.push_str(cur_completion)
                            }
                        }
                    }
                }
                _ => panic!("should not reach here: Incomplete Arm"),
            }
        }
        self
    }

    fn is_delete(&self) -> bool {
        if let Err(err) = self {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    if let ErrType::Completion { delete, .. } = err.err_type {
                        delete
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn func_incomplete_cast<F>(mut self, func: F, cur_completion: &str, delete: bool) -> Self
    where
        F: Fn(&mut ErrRes<'a, &str>),
    {
        if let Ok((rem, _)) = &self {
            if let Ok((rem, _)) = sp(rem) {
                if let Some(c) = rem.chars().next() {
                    if c == ',' || c == ']' || c == '}' {
                        return self;
                    }
                }
            } else {
                panic!("Unexpected behaviour")
            }
            // 这里已经是不完整的，所以直接传入空字符即可
            let mut err_res = ErrRes::from_error_kind("", nom::error::ErrorKind::Fail);
            func(&mut err_res);
            self = Err(nom::Err::Error(err_res));
        } else {
            self = self.func_cast(func, cur_completion, delete);
        }
        self
    }

    fn incomplete_cast(
        self,
        cur_str: &'a str,
        cur_completion: &str,
        json_type: JsonType,
        delete: bool,
    ) -> Self {
        self.func_incomplete_cast(
            |err| {
                let (rem, _) = err.base_err.errors.split_first().unwrap();
                // debug_println!("rem: {}", rem.0);
                let completion = if !delete {
                    cur_completion.to_string()
                } else {
                    String::new()
                };
                err.err_type = if rem.0.is_empty() {
                    ErrType::Completion {
                        delete,
                        completion,
                        json_type,
                    }
                } else {
                    ErrType::Failure
                };
                // debug_println!("Err: {:?}", err.err_type);
                err.err_str = Some(cur_str)
            },
            cur_completion,
            delete,
        )
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

#[allow(dead_code, unused)]
fn parse_any(i: &str) -> ParseRes<&str> {
    todo!()
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
    res.cast(i, "\"", JsonType::Str, false)
}

fn parse_key_value(i: &str) -> ParseRes<(&str, &str)> {
    // 临时用parse_string顶替一下，测试
    let res: ParseRes<(&str, &str)> = separated_pair(
        preceded(sp, parse_string),
        cut(preceded(sp, char(':'))),
        parse_string,
    )(i);
    res.cast(i, ",", JsonType::KeyVal, false)
}

fn parse_spec(i: &str) -> ParseRes<&str> {
    // 对于非一个一个字符的匹配，incomplete的语义是不一样的
    // 对于alt使用，如果不是最终的alt，一定要改写它的error_type
    let res: ParseRes<&str> = alt((
        tag("false"),
        tag("true"),
        tag("NaN"),
        tag("Null"),
        tag("Infinity"),
        tag("-Infinity"),
    ))(i);
    let spec_vec = [
        ("true", 1),
        ("false", 1),
        ("NaN", 2),
        ("Null", 1),
        ("Infinity", 1),
        ("-Infinity", 2),
    ];
    res.func_cast(
        |err_res| {
            err_res.err_str = Some(i);
            let completion = spec_vec
                .iter()
                .find_map(|(pattern, min_len)| {
                    if utils::is_prefix_with_min_length(pattern, i, *min_len) {
                        utils::complement_after(pattern, i)
                    } else {
                        None
                    }
                })
                .unwrap_or("");
            err_res.err_type = if completion.is_empty() {
                ErrType::Failure
            } else {
                // 这是最小的一级，所以不需要考虑append
                ErrType::Completion {
                    delete: false,
                    completion: completion.to_string(),
                    json_type: JsonType::Spec,
                }
            }
        },
        "",
        false,
    )
}

// 解析可能的科学计数尾部
fn parse_e_(input: &str) -> ParseRes<&str> {
    let parse_sign = opt(alt((char('-'), char('+'))));
    let parse_e = alt((char('e'), char('E')));
    let parse_tuple = tuple((parse_e, parse_sign, digit1));
    recognize(parse_tuple)(input)
}

// 解析基数，包括整数部分和可选的小数部分
fn parse_base_(input: &str) -> ParseRes<&str> {
    recognize(tuple((
        opt(char('-')),
        digit1,
        opt(preceded(char('.'), digit1)),
    )))(input)
}

fn parse_num(i: &str) -> ParseRes<&str> {
    let parse_tuple = tuple((parse_base_, opt(parse_e_)));
    recognize(parse_tuple)(i).incomplete_cast(i, "", JsonType::Num, true)
}

fn parse_arr(i: &str) -> ParseRes<&str> {
    let content = alt((parse_num, parse_spec, parse_string, parse_arr));
    let content_with_sp = tuple((sp, content, sp));
    let contents = separated_list0(char(','), content_with_sp);
    let match_tuple = tuple((char('['), contents, char(']')));
    recognize(match_tuple)(i).cast(i, "]", JsonType::Array, false)
}

#[allow(unused)]
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

#[allow(unused)]
macro_rules! quick_test_ok {
    ($input: expr, $func: ident, $eq_left: expr) => {{
        let output = $func($input);
        assert_eq!($eq_left, output);
    }};
}

#[cfg(test)]
mod test_string {
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

#[cfg(test)]
mod test_keyval {
    use super::*;

    #[test]
    fn test_keyval_incomplete() {
        quick_test_failed!(r#""laljfw""#, parse_key_value,
            ErrType::Completion {
                delete: false,
                completion: r#","#.to_string(),
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

#[cfg(test)]
mod test_boolean {
    use super::*;

    #[test]
    fn test_bool_ok() {
        quick_test_ok!("true", parse_spec, Ok(("", "true")));
        quick_test_ok!("false", parse_spec, Ok(("", "false")));
        quick_test_ok!("NaN", parse_spec, Ok(("", "NaN")));
        quick_test_ok!("Null", parse_spec, Ok(("", "Null")));
        quick_test_ok!("Infinity", parse_spec, Ok(("", "Infinity")));
        quick_test_ok!("-Infinity", parse_spec, Ok(("", "-Infinity")));
    }

    #[test]
    fn test_bool_incomplete() {
        quick_test_failed!("tr", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "ue".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("tr") => err_str
        );

        quick_test_failed!("fal", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "se".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("fal") => err_str
        );

        quick_test_failed!("Na", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "N".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("Na") => err_str
        );

        quick_test_failed!("N", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "ull".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("N") => err_str
        );

        quick_test_failed!("Inf", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "inity".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("Inf") => err_str
        );

        quick_test_failed!("-I", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "nfinity".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("-I") => err_str
        );
    }

    #[test]
    fn test_bool_invalid() {
        quick_test_failed!("-", parse_spec,
            ErrType::Failure => err_type,
            Some("-") => err_str
        );

        quick_test_failed!("tu", parse_spec,
            ErrType::Failure => err_type,
            Some("tu") => err_str
        );

        quick_test_failed!("", parse_spec,
            ErrType::Failure => err_type,
            Some("") => err_str
        );

        quick_test_failed!("fl", parse_spec,
            ErrType::Failure => err_type,
            Some("fl") => err_str
        );

        quick_test_failed!("Nau", parse_spec,
            ErrType::Failure => err_type,
            Some("Nau") => err_str
        );
    }
}

mod test_num {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_num_valid() {
        quick_test_ok!("0,", parse_num, Ok((",", "0")));
        quick_test_ok!("-0,", parse_num, Ok((",", "-0")));
        quick_test_ok!("123,", parse_num, Ok((",", "123")));
        quick_test_ok!("-123 ,", parse_num, Ok((" ,", "-123")));
        quick_test_ok!("12.34,", parse_num, Ok((",", "12.34")));
        quick_test_ok!("-12.34]", parse_num, Ok(("]", "-12.34")));
        quick_test_ok!("0.123}", parse_num, Ok(("}", "0.123")));
        quick_test_ok!("123e10  ]", parse_num, Ok(("  ]", "123e10")));
        quick_test_ok!("123E10,", parse_num, Ok((",", "123E10")));
        quick_test_ok!("123e+10  }", parse_num, Ok(("  }", "123e+10")));
        quick_test_ok!("123e-10 ,", parse_num, Ok((" ,", "123e-10")));
        quick_test_ok!("-123e-10,", parse_num, Ok((",", "-123e-10")));
        quick_test_ok!(
            "100000000000000000000000000,",
            parse_num,
            Ok((",", "100000000000000000000000000"))
        );
    }

    #[test]
    fn test_num_incomplete() {
        assert!(parse_num("1e").is_incomplete());
        assert!(parse_num("0").is_incomplete());
        assert!(parse_num("-123").is_incomplete());
        assert!(parse_num("0.123").is_incomplete());
        assert!(parse_num("0.").is_incomplete());
    }

    // #[test]
    // fn test_num_invalid() {
    //     assert!(parse_num("-").is_incomplete());
    //     assert!(parse_num(".123").is_invalid());
    //     assert!(parse_num("12.3.4").is_invalid());
    // }
}

mod test_array {
    #[allow(unused)]
    use super::*;

    #[test]
    fn test_arr_valid() {
        quick_test_ok!(
            r#"["apple", "banana", "cherry"]"#,
            parse_arr,
            Ok(("", r#"["apple", "banana", "cherry"]"#))
        );
        quick_test_ok!(
            r#"[1, 2, 3, 4, 5]"#,
            parse_arr,
            Ok(("", r#"[1, 2, 3, 4, 5]"#))
        );
        quick_test_ok!(
            r#"[true, false, false, true]"#,
            parse_arr,
            Ok(("", r#"[true, false, false, true]"#))
        );
        quick_test_ok!(
            r#"[[1, 2, 3], [4, 5, 6], ["seven", "eight", "nine"]]"#,
            parse_arr,
            Ok(("", r#"[[1, 2, 3], [4, 5, 6], ["seven", "eight", "nine"]]"#))
        );
    }
}

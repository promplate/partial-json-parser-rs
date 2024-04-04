use std::collections::HashMap;

use nom::bytes::complete::{escaped, tag, take, take_till, take_while, take_while1};
use nom::character::complete::{char, one_of};
use nom::combinator::{complete, cut};
use nom::error::{FromExternalError, ParseError, VerboseError};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum JsonValue<'a> {
    Str(&'a str),
    Boolean(bool),
    Num(f64),
    Array(Vec<JsonValue<'a>>),
    Object(HashMap<&'a str, JsonValue<'a>>),
}


struct ErrRes<I> {
    base_err: VerboseError<I>,
    pub completion: String,
    pub idx: usize,
}

impl <I> ErrRes<I> {
    // 对于不同的类型，需要不同的completion设置，所以
    fn simple_cast(base_err: VerboseError<I>) -> Self {
        Self {
            base_err,
            completion: String::default(),
            idx: usize::default(),
        }
    }
}

impl <I> ParseError<I> for ErrRes<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        ErrRes {
            base_err: VerboseError::from_error_kind(input, kind),
            completion: String::default(),
            idx: usize::default(),
        }
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.base_err =  VerboseError::append(input, kind, other.base_err);
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

fn test(i: &str) -> IResult<&str, &str, ErrRes<&str>>  {
    let res = preceded(
        char('\"'),
        terminated(
            escaped(
                take_while1(is_string_character),
                '\\',
                one_of("\"bfnrt\\"),
            ),
            char('\"'),
        ),
    )(i);
}

fn string(i: &str) -> IResult<&str, &str, ErrRes<&str>> {
    preceded(
        char('\"'),
        terminated(
            escaped(
                take_while1(is_string_character),
                '\\',
                one_of("\"bfnrt\\"),
            ),
            char('\"'),
        ),
    )(i)
}

// fn key_value(&mut self, i: &str) -> IResult<&str, &str> {
//     separated_pair(, sep, second)
// }

// fn complete_num(&mut self, input: &str) -> IResult<&str, &str> {

// }


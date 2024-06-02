use nom::{
    branch::alt,
    character::complete::{char, digit1},
    combinator::{opt, recognize},
    sequence::{preceded, tuple},
};

use super::{ErrCast, JsonType, ParseRes};

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

pub fn parse_num(i: &str) -> ParseRes<&str> {
    let parse_tuple = tuple((parse_base_, opt(parse_e_)));
    let res = recognize(parse_tuple)(i).incomplete_cast(i, "", JsonType::Num, true);
    if res.is_incomplete() {
        return res.try_to_failure();
    }
    res
}

#[cfg(test)]
mod test_num {
    use crate::quick_test_ok;

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

    #[test]
    fn test_num_invalid() {
        assert!(parse_num("-").is_incomplete());
        assert!(parse_num(".123").is_invalid());
        assert!(parse_num("12.3.4").is_invalid());
    }
}

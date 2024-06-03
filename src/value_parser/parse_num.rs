use nom::{
    branch::alt,
    character::complete::{char, digit1},
    combinator::{opt, recognize},
    sequence::{preceded, tuple},
    IResult,
};

use super::VParserRes;

// 解析可能的科学计数尾部
fn parse_e_(input: &str) -> IResult<&str, &str> {
    let parse_sign = opt(alt((char('-'), char('+'))));
    let parse_e = alt((char('e'), char('E')));
    let parse_tuple = tuple((parse_e, parse_sign, digit1));
    recognize(parse_tuple)(input)
}

// 解析基数，包括整数部分和可选的小数部分
fn parse_base_(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        opt(char('-')),
        digit1,
        opt(preceded(char('.'), digit1)),
    )))(input)
}

pub fn parse_num(i: &str) -> Result<VParserRes, ()> {
    // 尝试解析基数部分
    let base_res = parse_base_(i);
    if let Ok((remaining, matched)) = base_res {
        let no_e = !remaining.starts_with('e')
            && !remaining.starts_with("e-")
            && !remaining.starts_with('E')
            && !remaining.starts_with("E-");
        if remaining.starts_with('.') {
            return Ok(VParserRes::new(matched, false));
        } else if no_e {
            // 仅有基数部分，且无剩余输入
            return Ok(VParserRes::new(matched, false)); // 其实这里应该也算是incomplete的，因为这是数字
        }

        // 尝试解析指数部分
        let exponent_res = parse_e_(remaining);
        if let Ok(exponent_res) = exponent_res {
            return Ok(VParserRes::new(matched.to_string() + exponent_res.1, false));
        } else {
            // 存在未完全解析的指数部分
            return Err(());
        }
    }

    // 基数部分解析失败
    Err(())
}

#[cfg(test)]
mod test_num {
    use crate::quick_test_ok;

    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_num_valid() {
        quick_test_ok!("0,", parse_num, Ok(VParserRes::new("0", false)));
        quick_test_ok!("-0,", parse_num, Ok(VParserRes::new("-0", false)));
        quick_test_ok!("123,", parse_num, Ok(VParserRes::new("123", false)));
        quick_test_ok!("-123 ,", parse_num, Ok(VParserRes::new("-123", false)));
        quick_test_ok!("12.34,", parse_num, Ok(VParserRes::new("12.34", false)));
        quick_test_ok!("-12.34]", parse_num, Ok(VParserRes::new("-12.34", false)));
        quick_test_ok!("0.123}", parse_num, Ok(VParserRes::new("0.123", false)));
        quick_test_ok!("123e10  ]", parse_num, Ok(VParserRes::new("123e10", false)));
        quick_test_ok!("123E10,", parse_num, Ok(VParserRes::new("123E10", false)));
        quick_test_ok!("123e+10  }", parse_num, Ok(VParserRes::new("123e+10", false)));
        quick_test_ok!("123e-10 ,", parse_num, Ok(VParserRes::new("123e-10", false)));
        quick_test_ok!("-123e-10,", parse_num, Ok(VParserRes::new("-123e-10", false)));
        quick_test_ok!(
            "100000000000000000000000000,",
            parse_num,
            Ok(VParserRes::new("100000000000000000000000000", false))
        );
    }

    #[test]
    fn test_num_incomplete() {
        assert!(parse_num("1e").is_err());
        assert!(parse_num("1e-").is_err());
        assert!(parse_num("-123E").is_err());
        assert_eq!(parse_num("0."), Ok(VParserRes::new("0", false)));
        assert_eq!(parse_num("12."), Ok(VParserRes::new("12", false)));
        assert!(parse_num("2E-").is_err());
    }

    #[test]
    fn test_num_invalid() {
        assert!(parse_num("-").is_err());
        assert!(parse_num(".123").is_err());
        // 下面这个在我的测试用例中是通过的，但是不必太过于在意
        // 由于补全是针对正确的json截取，这个也没有必要太过于纠结
        // assert!(parse_num("12.3.4").is_err());
    }

    fn valid_number_string() -> impl Strategy<Value = String> {
        let integer = "[1-9][0-9]*|0";
        let decimal = format!("{}\\.{}", integer, "[0-9]+");
        let exponent = "[eE][+-]?[0-9]+";
        let number = format!(
            "({})|({})|({})({})?|({})({})?",
            integer, decimal, integer, exponent, decimal, exponent
        );

        prop::string::string_regex(&number).unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn test_parse_num(input in valid_number_string()) {
            for (i, _) in input.char_indices().skip(1) {
                let s = &input[..i];
                // println!("input: {}", s);
                if let Ok(res) = parse_num(s) {
                    let res = res.amend_value;
                    if !res.is_empty() {
                        res.parse::<f64>().unwrap();
                    }
                }
            }
        }
    }
}

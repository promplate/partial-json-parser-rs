use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;

use crate::utils;

use super::VParserRes;

use super::parse_string::sp;

pub fn parse_spec(i: &str) -> Result<VParserRes, ()> {
    // 对于非一个一个字符的匹配，incomplete的语义是不一样的
    // 对于alt使用，如果不是最终的alt，一定要改写它的error_type
    let res: IResult<&str, &str> = alt((
        tag("false"),
        tag("true"),
        tag("NaN"),
        tag("null"),
        tag("Infinity"),
        tag("-Infinity"),
    ))(i);
    let spec_vec = [
        ("true", 1),
        ("false", 1),
        ("NaN", 1),
        ("null", 1),
        ("Infinity", 1),
        ("-Infinity", 2),
    ];

    let completion = spec_vec.iter().find_map(|(pattern, min_len)| {
        if utils::is_prefix_with_min_length(pattern, i, *min_len) {
            utils::complement_after(pattern, i)
        } else {
            None
        }
    });

    if let Some(cmpl) = completion {
        Ok(VParserRes::new(i.to_string() + cmpl, false))
    } else if res.is_ok() && completion.is_none() {
        Ok(VParserRes::new(res.unwrap().1, true))
    } else {
        Err(())
    }
}

#[cfg(test)]
mod test_spec {
    use std::cmp::min;

    use super::parse_spec;

    #[test]
    fn test_spec() {
        let spec_vec = [
            ("true", 1),
            ("false", 1),
            ("NaN", 1),
            ("null", 1),
            ("Infinity", 1),
            ("-Infinity", 2),
        ];

        for (s, min_len) in spec_vec {
            for (idx, _) in s.char_indices().skip(min_len - 1) {
                println!("{}, {}", idx, &s[..(idx+1)]);
                let res = parse_spec(&s[..(idx+1)]).unwrap().amend_value;
                // println!("{}", res);
                assert_eq!(s, res);
            }
        }

        assert!(parse_spec("-").is_err())
    }
}

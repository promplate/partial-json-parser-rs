use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;

use crate::utils;

use super::VParserRes;


#[allow(unused)]
pub fn parse_spec(i: &str) -> Result<VParserRes, ()> {
    let res: IResult<&str, &str> = alt((
        tag("false"),
        tag("true"),
        tag("NaN"),
        tag("null"),
        tag("Infinity"),
        tag("-Infinity"),
    ))(i);
    let spec_vec: [(&str, usize); 6] = [
        ("true", 1),
        ("false", 1),
        ("NaN", 1),
        ("null", 1),
        ("Infinity", 1),
        ("-Infinity", 2),
    ];

    shared(&spec_vec, res, i)
}

#[inline]
pub fn shared(
    spec_vec: &[(&str, usize)],
    res: IResult<&str, &str>,
    i: &str,
) -> Result<VParserRes, ()> {
    let completion = spec_vec.iter().find_map(|(pattern, min_len)| {
        if utils::is_prefix_with_min_length(pattern, i, *min_len) {
            utils::complement_after(pattern, i)
        } else {
            None
        }
    });

    if let Some(cmpl) = completion {
        Ok(VParserRes::new(i.to_string() + cmpl, false))
    } else if let Ok(res) = res {
        if completion.is_none() {
            Ok(VParserRes::new(res.1, true))
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

pub fn parse_bool(i: &str) -> Result<VParserRes, ()> {
    let res: IResult<&str, &str> = alt((tag("false"), tag("true")))(i);
    let spec_vec = [("true", 1), ("false", 1)];

    shared(&spec_vec, res, i)
}

pub fn parse_nan(i: &str) -> Result<VParserRes, ()> {
    let res: IResult<&str, &str> = tag("NaN")(i);
    let spec_vec = [("NaN", 1)];

    shared(&spec_vec, res, i)
}

pub fn parse_null(i: &str) -> Result<VParserRes, ()> {
    let res: IResult<&str, &str> = tag("null")(i);
    let spec_vec = [("null", 1)];

    shared(&spec_vec, res, i)
}

pub fn parse_infinity(i: &str) -> Result<VParserRes, ()> {
    let res: IResult<&str, &str> = tag("Infinity")(i);
    let spec_vec = [("Infinity", 1)];

    shared(&spec_vec, res, i)
}

pub fn parse_ninfinity(i: &str) -> Result<VParserRes, ()> {
    let res: IResult<&str, &str> = tag("-Infinity")(i);
    let spec_vec = [("-Infinity", 2)];

    shared(&spec_vec, res, i)
}

#[cfg(test)]
mod test_spec {

    use crate::value_parser::parse_spec::{parse_bool, parse_infinity, parse_nan, parse_ninfinity};

    use super::*;

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
                println!("{}, {}", idx, &s[..(idx + 1)]);
                let res = parse_spec(&s[..(idx + 1)]).unwrap().amend_value;
                // println!("{}", res);
                assert_eq!(s, res);
            }
        }

        assert!(parse_spec("-").is_err())
    }

    #[test]
    fn test_specs() {
        let spec_vec = [
            ("true", 1),
            ("false", 1),
            ("NaN", 1),
            ("null", 1),
            ("Infinity", 1),
            ("-Infinity", 2),
        ];

        for (vec_idx, (s, min_len)) in spec_vec.iter().enumerate() {
            for (idx, _) in s.char_indices().skip(min_len - 1) {
                println!("{}, {}", idx, &s[..(idx + 1)]);
                let res = if vec_idx == 0 || vec_idx == 1 {
                    parse_bool(&s[..(idx + 1)]).unwrap().amend_value
                } else if vec_idx == 2 {
                    parse_nan(&s[..(idx + 1)]).unwrap().amend_value
                } else if vec_idx == 3 {
                    parse_null(&s[..(idx + 1)]).unwrap().amend_value
                } else if vec_idx == 4 {
                    parse_infinity(&s[..(idx + 1)]).unwrap().amend_value
                } else {
                    parse_ninfinity(&s[..(idx + 1)]).unwrap().amend_value
                };
                // println!("{}", res);
                assert_eq!(*s, res);
            }
        }

        assert!(parse_ninfinity("-").is_err())
    }
}

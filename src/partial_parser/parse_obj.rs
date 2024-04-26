use nom::character::complete::char;
use nom::{
    combinator::{cut, recognize},
    multi::separated_list0,
    sequence::{preceded, separated_pair, tuple},
};

use crate::with_sp;

use super::parse_any::parse_any;
use super::ErrType;
use super::{
    parse_string::{parse_key, sp},
    ErrCast, JsonType, ParseRes,
};

fn sep(i: &str) -> ParseRes<&str> {
    let mut res = recognize(with_sp!(char(':')))(i);
    if let Err(nom::Err::Error(ref mut err) | nom::Err::Failure(ref mut err)) = res {
        err.err_type = ErrType::Completion { delete: true, json_type: JsonType::KeyVal, completion: String::from("") };
    }
    println!("sep: {:#?}", res);
    res
}

fn parse_key_value(i: &str) -> ParseRes<(&str, &str)> {
    let res: ParseRes<(&str, &str)> = separated_pair(preceded(sp, parse_key), sep, parse_any)(i);
    println!("key_val: {:#?}", res);
    let is_delete = res.is_delete();
    res.cast(i, "", JsonType::KeyVal, is_delete)
}

pub(super) fn parse_obj(i: &str) -> ParseRes<&str> {
    let content = parse_key_value;
    let separator = tuple((sp, char(','), sp));
    let contents = separated_list0(separator, cut(content));
    let match_tuple = tuple((with_sp!(char('{')), contents, with_sp!(char('}'))));
    let res = recognize(match_tuple)(i);
    let res = res.cast(i, "}", JsonType::Object, false);
    if res.is_incomplete() {
        return res.try_to_failure();
    }
    res
}

#[cfg(test)]
mod test_obj {
    use crate::quick_test_ok;

    #[allow(unused)]
    use super::*;

    #[test]
    fn test_obj_valid() {
        quick_test_ok!(
            r#"{
                "name": "John",
                "age": 30,
                "city": "New York"
              }"#,
            parse_obj,
            Ok((
                "",
                r#"{
                "name": "John",
                "age": 30,
                "city": "New York"
              }"#
            ))
        );

        assert!(parse_obj(
            r#"{
            "name": "John",
            "age": 30,
            "city": "New York",
            "hobbies": ["reading", "gaming", "cooking"]
        }"#
        )
        .is_ok());

        assert!(parse_obj(
            r#"{
            "name": "John",
            "age": 30,
            "city": "New York",
            "address": {
              "street": "123 Main St",
              "city": "New York",
              "state": "NY",
              "zip": "10001"
            }
          }"#
        )
        .is_ok());
    }

    #[test]
    fn test_obj_completion() {
        let res = parse_obj(
            r#"{
            "name": "John",
            "age": 30,
            "city": "New York","#,
        );
        // println!("{:#?}", res);
        println!("{:#?}", res.completion())
    }

    #[test]
    fn test_obj_completion1() {
        let res = parse_obj(
            r#"{
                "person": {
                  "name": "John Doe",
                  "age": 30,
                  "is_student": false
                },
                "erd"
                "#,
        );
        println!("{:#?}", res);
        println!("{:#?}", res.completion())
    }
}

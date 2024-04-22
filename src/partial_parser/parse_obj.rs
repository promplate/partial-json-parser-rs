use nom::character::complete::char;
use nom::{
    combinator::{cut, recognize},
    multi::separated_list0,
    sequence::{preceded, separated_pair, tuple},
};

use crate::with_sp;

use super::parse_any::parse_any;
use super::{
    parse_string::{parse_key, sp},
    ErrCast, JsonType, ParseRes,
};

fn parse_key_value(i: &str) -> ParseRes<(&str, &str)> {
    // 临时用parse_string顶替一下，测试
    let res: ParseRes<(&str, &str)> =
        separated_pair(preceded(sp, parse_key), cut(with_sp!(char(':'))), parse_any)(i);
    // debug_println!("key_val: {:#?}", res);
    res.cast(i, ",", JsonType::KeyVal, false)
}

pub(super) fn parse_obj(i: &str) -> ParseRes<&str> {
    let content = parse_key_value;
    let separator = tuple((sp, char(','), sp));
    let contents = separated_list0(separator, content);
    let match_tuple = tuple((with_sp!(char('{')), contents, with_sp!(char('}'))));
    recognize(match_tuple)(i).cast(i, "}", JsonType::Object, false)
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
}

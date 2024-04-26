use crate::with_sp;
use nom::character::complete::char;
use nom::combinator::cut;
use nom::{branch::alt, combinator::recognize, multi::separated_list0, sequence::tuple};

use super::{parse_any::parse_any, ErrCast};
use super::{
    parse_num,
    parse_spec::parse_spec,
    parse_string::{parse_string, sp},
    JsonType, ParseRes,
};

pub(super) fn parse_arr(i: &str) -> ParseRes<&str> {
    let content = parse_any;
    let separator = tuple((sp, char(','), sp));
    let contents = separated_list0(separator, cut(content));
    let match_tuple = tuple((with_sp!(char('[')), cut(contents), with_sp!(char(']'))));
    recognize(match_tuple)(i).cast(i, "]", JsonType::Array, false)
}

#[cfg(test)]
mod test_array {
    use crate::quick_test_ok;

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

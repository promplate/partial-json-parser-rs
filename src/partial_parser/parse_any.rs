use nom::branch::alt;

use super::{
    parse_array::parse_arr, parse_num::parse_num, parse_obj::parse_obj, parse_spec::parse_spec,
    parse_string::parse_string, ParseRes,
};

pub(super) fn parse_any(i: &str) -> ParseRes<&str> {
    alt((parse_arr, parse_obj, parse_num, parse_spec, parse_string))(i)
}

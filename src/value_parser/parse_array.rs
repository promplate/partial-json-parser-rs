use nom::{
    IResult,
    bytes::complete::{tag, take_until},
    character::complete::char,
    sequence::delimited,
};

use super::VParserRes;


pub fn parse_array(s: &str) -> Result<VParserRes, ()> {
    let (s, _) = super::sp(s).unwrap();

    if s.starts_with('[') {
        let res: IResult<&str, &str> = delimited(char('['), take_until("]"), char(']'))(s);
        if let Ok((_, _)) = res {
            Ok(VParserRes::new("", false))
        } else {
            Ok(VParserRes::new("", true).set_stack_recover(true))
        }
    } else {
        Err(())
    }
}
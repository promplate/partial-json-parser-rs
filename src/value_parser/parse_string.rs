use std::f32::consts::E;

use nom::{
    bytes::complete::take_while, IResult
};
use super::VParserRes;
use crate::parser::{ EscapeCnt, CharType };

fn is_space(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\r' || c == '\n'
}

pub fn sp(i: &str) -> IResult<&str, &str> {
    take_while(is_space)(i)
}

fn is_string_character(c: char) -> bool {
    //FIXME: should validate unicode character
    println!("{}", c);
    c != '"' && c != '\\'
}

pub fn parse_string(i: &str) -> Result<VParserRes, ()> {
    let (s, _) = sp(i).map_err(|_| ())?;
    if s.is_empty() {
        return Err(());
    }

    let mut first = true;
    let mut need_cmpl = true;
    let mut esc_cnt = EscapeCnt::new();
    let mut amend_value = String::new();
    let mut last_esc = None;

    for (idx, c) in s.char_indices() {
        if first && c != '"' {
            return Err(());
        } else if first && c == '"' {
            first = false;
            continue;
        }
        let char_type = esc_cnt.input(c);
        if char_type == CharType::Quotation {
            need_cmpl = false;
            break;
        } else if char_type == CharType::Escape {
            last_esc = Some(idx);
        }
    }
    
    if need_cmpl {
        if esc_cnt.cnt() == 1 {
            // 这时候需要从最后一个转义符号恢复
            amend_value.push_str(&s[..last_esc.expect("Should have value")]);
        } else {
            amend_value.push_str(s);
        }
        amend_value.push('"')
    }
    Ok(VParserRes {
        amend_value
    })
}

#[cfg(test)]
mod test {

    use crate::value_parser::parse_string::parse_string;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn test_str(test_s in ".*") {
            let test_s = serde_json::to_string(&test_s).unwrap();
            for (idx, _) in test_s.char_indices() {
                if idx == 0 {
                    continue;
                }
                let s = parse_string(&test_s[..idx]).unwrap().amend_value;
                if !s.is_empty() {
                    assert!(serde_json::from_str::<String>(&s).is_ok());
                }
            }
        }
    }

    #[test]
    fn test_cases() {
        let test_vec = vec![r#""hjkhjk"#, r#""\u00"#, r#""\u0000\b"#];
        for i in test_vec {
            let s = parse_string(i).unwrap().amend_value;
            if !s.is_empty() {
                println!("{}", s);
                assert!(serde_json::from_str::<String>(&s).is_ok());
            }
        }
    }
} 

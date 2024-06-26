use super::VParserRes;
use crate::parser::{CharType, EscapeCnt};
use nom::{bytes::complete::take_while, IResult};

fn is_space(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\r' || c == '\n'
}

pub fn sp(i: &str) -> IResult<&str, &str> {
    take_while(is_space)(i)
}


pub fn parse_string(i: &str) -> Result<VParserRes, ()> {
    let (s, _) = sp(i).map_err(|_| ())?;
    if s.is_empty() {
        return Err(());
    }

    let mut first = true;
    let mut need_cmpl = true;
    let mut end_of_s = 0;
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
            end_of_s = idx;
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
    } else {
        // 如果不需要补全，直接返回相应的字符串
        amend_value.push_str(&s[..=end_of_s])
    }
    Ok(VParserRes::new(amend_value, !need_cmpl))
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
        let test_vec = vec![
            r#""hjkhjk"#,
            r#""\u00"#,
            r#""\u0000\b"#,
            r#""abc""#,
            r#""abc\"\"""#,
        ];
        for i in test_vec {
            let s = parse_string(i).unwrap().amend_value;
            println!("{}", s);
            assert!(serde_json::from_str::<String>(&s).is_ok());
        }
    }
}

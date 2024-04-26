use nom::{branch::alt, bytes::complete::tag};

use crate::utils;

use super::{ErrCast, ErrType, JsonType, ParseRes};

pub(super) fn parse_spec(i: &str) -> ParseRes<&str> {
    // 对于非一个一个字符的匹配，incomplete的语义是不一样的
    // 对于alt使用，如果不是最终的alt，一定要改写它的error_type
    let res: ParseRes<&str> = alt((
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
    let res = res.func_cast(
        |err_res| {
            err_res.err_str = Some(i);
            let completion = spec_vec
                .iter()
                .find_map(|(pattern, min_len)| {
                    if utils::is_prefix_with_min_length(pattern, i, *min_len) {
                        utils::complement_after(pattern, i)
                    } else {
                        None
                    }
                })
                .unwrap_or("");
            err_res.err_type = if completion.is_empty() {
                ErrType::Failure
            } else {
                // 这是最小的一级，所以不需要考虑append
                ErrType::Completion {
                    delete: false,
                    completion: completion.to_string(),
                    json_type: JsonType::Spec,
                }
            }
        },
        i,
        "",
        false,
    );
    if res.is_incomplete() {
        return res.try_to_failure();
    }
    res
}

#[cfg(test)]
mod test_spec {
    use crate::{quick_test_failed, quick_test_ok};

    use super::*;

    #[test]
    fn test_spec_ok() {
        quick_test_ok!("true", parse_spec, Ok(("", "true")));
        quick_test_ok!("false", parse_spec, Ok(("", "false")));
        quick_test_ok!("NaN", parse_spec, Ok(("", "NaN")));
        quick_test_ok!("null", parse_spec, Ok(("", "null")));
        quick_test_ok!("Infinity", parse_spec, Ok(("", "Infinity")));
        quick_test_ok!("-Infinity", parse_spec, Ok(("", "-Infinity")));
    }

    #[test]
    fn test_spec_incomplete() {
        quick_test_failed!("tr", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "ue".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("tr") => err_str
        );

        quick_test_failed!("fal", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "se".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("fal") => err_str
        );

        quick_test_failed!("Na", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "N".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("Na") => err_str
        );

        quick_test_failed!("n", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "ull".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("n") => err_str
        );

        quick_test_failed!("Inf", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "inity".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("Inf") => err_str
        );

        quick_test_failed!("-I", parse_spec,
            ErrType::Completion {
                delete: false,
                completion: "nfinity".to_string(),
                json_type: JsonType::Spec,
            } => err_type,
            Some("-I") => err_str
        );
    }

    #[test]
    fn test_spec_invalid() {
        quick_test_failed!("-", parse_spec,
            ErrType::Failure => err_type,
            Some("-") => err_str
        );

        quick_test_failed!("tu", parse_spec,
            ErrType::Failure => err_type,
            Some("tu") => err_str
        );

        quick_test_failed!("", parse_spec,
            ErrType::Failure => err_type,
            Some("") => err_str
        );

        quick_test_failed!("fl", parse_spec,
            ErrType::Failure => err_type,
            Some("fl") => err_str
        );

        quick_test_failed!("Nau", parse_spec,
            ErrType::Failure => err_type,
            Some("Nau") => err_str
        );
    }
}

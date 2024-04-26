use nom::{
    branch::alt, error::{ParseError, VerboseError}, IResult
};

mod parse_any;
mod parse_array;
mod parse_num;
mod parse_obj;
mod parse_spec;
mod parse_string;
use crate::{debug_println, utils::{exclude_substring, remove_trailing_comma_and_whitespace}};

use self::{parse_array::parse_arr, parse_obj::parse_obj, parse_string::sp};

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub enum JsonType {
    #[default]
    Any,
    Str,
    Spec,
    Num,
    KeyVal,
    Array,
    Object,
}

#[derive(Debug, PartialEq, Eq, Default)]
enum ErrType {
    #[default]
    Failure,
    Completion {
        delete: bool,
        json_type: JsonType,
        completion: String,
    },
}

#[derive(Debug, PartialEq)]
struct ErrRes<'a, I> {
    pub base_err: VerboseError<I>,
    pub err_type: ErrType,
    pub err_str: Option<&'a str>,
}

impl<'a, I> ErrRes<'a, I> {
    fn simple_cast(base_err: VerboseError<I>) -> Self {
        Self {
            base_err,
            err_type: ErrType::default(),
            err_str: None,
        }
    }
}

impl<'a, I> ParseError<I> for ErrRes<'a, I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        ErrRes::simple_cast(VerboseError::from_error_kind(input, kind))
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.base_err = VerboseError::append(input, kind, other.base_err);
        other
    }
}

type ParseRes<'a, O> = IResult<&'a str, O, ErrRes<'a, &'a str>>;

trait ErrCast<'a> {
    fn cast(
        self,
        cur_str: &'a str,
        cur_completion: &str,
        json_type: JsonType,
        delete: bool,
    ) -> Self;
    fn func_cast<F>(self, func: F, cur_str: &'a str, cur_completion: &str, delete: bool) -> Self
    where
        F: Fn(&mut ErrRes<'a, &str>);
    fn func_incomplete_cast<F>(
        self,
        func: F,
        cur_str: &'a str,
        cur_completion: &str,
        delete: bool,
    ) -> Self
    where
        F: Fn(&mut ErrRes<'a, &str>);
    fn incomplete_cast(
        self,
        cur_str: &'a str,
        cur_completion: &str,
        json_type: JsonType,
        delete: bool,
    ) -> Self;
    fn try_to_failure(self) -> Self;
    fn is_invalid(&self) -> bool;
    fn is_incomplete(&self) -> bool;
    fn is_delete(&self) -> bool;
    fn completion(&self) -> Option<String>;
}

fn remove_sp(i: &str) -> &str {
    let (rem, _) = sp(i).unwrap();
    rem
}

impl<'a, O> ErrCast<'a> for ParseRes<'a, O> {
    fn cast(
        self,
        cur_str: &'a str,
        cur_completion: &str,
        json_type: JsonType,
        delete: bool,
    ) -> Self {
        self.func_cast(
            |err| {
                let (rem, _) = err.base_err.errors.split_first().unwrap();
                // debug_println!("rem: {}", rem.0);
                let completion = if !delete {
                    cur_completion.to_string()
                } else {
                    String::new()
                };
                err.err_type = if remove_sp(rem.0).is_empty() {
                    ErrType::Completion {
                        delete,
                        completion,
                        json_type,
                    }
                } else {
                    ErrType::Failure
                };
                err.err_str = Some(cur_str)
            },
            cur_str,
            cur_completion,
            delete,
        )
    }

    fn is_invalid(&self) -> bool {
        let mut invalid = false;
        if let Err(ref err) = self {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    if err.err_str.is_none() {
                        let (rem, _) = err.base_err.errors.split_first().unwrap();
                        invalid = !remove_sp(rem.0).is_empty();
                    } else {
                        invalid = err.err_type == ErrType::Failure;
                    }
                }
                _ => panic!("should not reach here: Incomplete Arm"),
            }
        }
        invalid
    }

    // 这里其实是有点问题的，对于非一个字符一个字符的匹配，它得到的结论是错误的
    // 比如说对于tag来说
    fn is_incomplete(&self) -> bool {
        self.is_err() && !self.is_invalid()
    }

    fn func_cast<F>(mut self, func: F, cur_str: &'a str, cur_completion: &str, delete: bool) -> Self
    where
        F: Fn(&mut ErrRes<'a, &str>),
    {
        if let Err(ref mut err) = self {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    if err.err_str.is_none() {
                        func(err);
                    } else {
                        // 不是第一次发生错误，需要对completion做补足
                        if let ErrType::Completion {
                            ref mut completion,
                            delete: ref mut delete_,
                            ..
                        } = err.err_type
                        {
                            *delete_ = delete;
                            if !delete {
                                completion.push_str(cur_completion)
                            }
                            if delete {
                                err.err_str = Some(cur_str)
                            }
                        }
                    }
                }
                _ => panic!("should not reach here: Incomplete Arm"),
            }
        }
        self
    }

    fn is_delete(&self) -> bool {
        if let Err(err) = self {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    if let ErrType::Completion { delete, .. } = err.err_type {
                        delete
                    } else {
                        false
                    }
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn func_incomplete_cast<F>(
        mut self,
        func: F,
        cur_str: &'a str,
        cur_completion: &str,
        delete: bool,
    ) -> Self
    where
        F: Fn(&mut ErrRes<'a, &str>),
    {
        if let Ok((rem, _)) = &self {
            if let Ok((rem, _)) = sp(rem) {
                if let Some(c) = rem.chars().next() {
                    if c == ',' || c == ']' || c == '}' {
                        return self;
                    }
                }
            } else {
                panic!("Unexpected behaviour")
            }
            // 这里已经是不完整的，所以直接传入空字符即可
            let mut err_res = ErrRes::from_error_kind("", nom::error::ErrorKind::Fail);
            func(&mut err_res);
            self = Err(nom::Err::Error(err_res));
        } else {
            self = self.func_cast(func, cur_str, cur_completion, delete);
        }
        self
    }

    fn incomplete_cast(
        self,
        cur_str: &'a str,
        cur_completion: &str,
        json_type: JsonType,
        delete: bool,
    ) -> Self {
        self.func_incomplete_cast(
            |err| {
                let (rem, _) = err.base_err.errors.split_first().unwrap();
                // debug_println!("rem: {}", rem.0);
                let completion = if !delete {
                    cur_completion.to_string()
                } else {
                    String::new()
                };
                err.err_type = if remove_sp(rem.0).is_empty() {
                    ErrType::Completion {
                        delete,
                        completion,
                        json_type,
                    }
                } else {
                    ErrType::Failure
                };
                // debug_println!("Err: {:?}", err.err_type);
                err.err_str = Some(cur_str)
            },
            cur_str,
            cur_completion,
            delete,
        )
    }

    fn completion(&self) -> Option<String> {
        if let Err(err) = self {
            match err {
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    if let ErrType::Completion { ref completion, .. } = err.err_type {
                        return Some(completion.clone());
                    }
                }
                _ => panic!("should not reach here: Incomplete Arm"),
            }
        }
        None
    }

    fn try_to_failure(self) -> Self {
        if let Err(err) = self {
            if let nom::Err::Error(err) = err {
                return Err(nom::Err::Failure(err));
            }
            Err(err)
        } else {
            self
        }
    }
    
}


fn complete(i: &str) -> Result<String, String> {
    let res = alt((parse_obj, parse_arr))(i);
    println!("comp res: {:#?}", res);
    if let Ok((_, res_str)) = res {
        Ok(res_str.to_string())
    } else if let Err(nom::Err::Error(err) | nom::Err::Failure(err)) = res {
        let comp_string = if let ErrType::Completion { completion, .. } = err.err_type {
            completion
        } else {
            String::default()
        };
        let mut res_string = if let Some(err_str) = err.err_str {
            exclude_substring(i, err_str)
        } else {
            String::from(i)
        };
        // 需要对最终的结果进行修饰，删除可能的逗号
        remove_trailing_comma_and_whitespace(&mut res_string);
        res_string.push_str(&comp_string);
        
        Ok(res_string)
    } else {
        return Err(String::from("Unexpected behaviour"));
    }
    // res
    // .map(|(_, out)| out)
    // .map_err(|_| String::from("Error"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_obj_completion1() {
        let res = complete(
            r#"{
                "person": {
                  "name": "John Doe",
                  "age": 30,
                  "is_student": false
                }
                "#,
        );
        println!("complete: {:#?}", res);
    }
}
use nom::{
    error::{ParseError, VerboseError},
    IResult,
};

mod parse_num;
mod parse_spec;
mod parse_string;

pub use parse_num::parse_num;
pub use parse_spec::{parse_bool, parse_infinity, parse_nan, parse_ninfinity, parse_null};
pub use parse_string::{parse_string, sp};

#[derive(Debug, PartialEq, Eq)]
pub struct VParserRes {
    amend_value: String,
    is_complete: bool,
}

impl VParserRes {
    fn new(amend_value: impl ToString, is_complete: bool) -> VParserRes {
        VParserRes {
            amend_value: amend_value.to_string(),
            is_complete,
        }
    }

    pub fn amend_value(&self) -> &String {
        &self.amend_value
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }
}

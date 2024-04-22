#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            print!($($arg)*);
        }
    };
}

pub fn complement_after<'a>(full: &'a str, part: &'a str) -> Option<&'a str> {
    full.find(part).map(|index| &full[index + part.len()..])
}

pub fn is_prefix_with_min_length(str1: &str, str2: &str, min_length: usize) -> bool {
    str1.starts_with(str2) && str2.len() >= min_length
}

#[macro_export]
#[allow(unused)]
macro_rules! quick_test_failed {
    ($input: expr, $func: ident, $($eq_left: expr => $field: ident), +) => {{
        let output = $func($input);
        match output {
            Err(nom::Err::Error(err)) | Err(nom::Err::Failure(err)) => {
                $(
                    assert_eq!($eq_left, err.$field);
                )+
            }
            _ => panic!("Output is ok or completion!"),
        }
    }};
}

#[macro_export]
#[allow(unused)]
macro_rules! quick_test_ok {
    ($input: expr, $func: ident, $eq_left: expr) => {{
        let output = $func($input);
        assert_eq!($eq_left, output);
    }};
}

#[macro_export]
macro_rules! with_sp {
    ($input: expr) => {
        tuple((sp, $input, sp))
    };
}

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

fn is_subslice(a: &str, b: &str) -> bool {
    let a_start = a.as_ptr() as usize;
    let a_end = a_start + a.len();
    let b_start = b.as_ptr() as usize;
    let b_end = b_start + b.len();

    b_start >= a_start && b_end <= a_end
}

pub fn exclude_substring(a: &str, b: &str) -> String {
    if !is_subslice(a, b) {
        panic!("b is not a sub slice of a");
    }
    let b_start = b.as_ptr() as usize - a.as_ptr() as usize;
    let b_end = b_start + b.len();

    let mut result = String::new();
    result.push_str(&a[..b_start]); // 添加b之前的部分
    result.push_str(&a[b_end..]);   // 添加b之后的部分
    result
}

pub fn remove_trailing_comma_and_whitespace(input: &mut String) {
    // 使用闭包来定义匹配模式
    let chars_to_trim: &[_] = &[' ', ',', '\t', '\r', '\n'];
    *input = input.trim_end_matches(chars_to_trim).to_string();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn test_not_subslice() {
        exclude_substring("djkj", "dj");
    }

    #[test]
    fn test_subslice_eq() {
        let a = "Hello World!";
        let b = &a[5..];
        assert_eq!(exclude_substring(a, b), "Hello");
    }

    #[test]
    fn test_trailing_comma_and_whitespace() {
        let mut example1 = "Hello, world,    ".to_string();
        let mut example2 = "Hello, world".to_string();
        let mut example3 = "Hello, world, ".to_string();
        let mut example4 = "Hello, world\n, ".to_string();
        let mut example5 = "Hello, world\t, ".to_string();
    
        remove_trailing_comma_and_whitespace(&mut example1);
        remove_trailing_comma_and_whitespace(&mut example2);
        remove_trailing_comma_and_whitespace(&mut example3);
        remove_trailing_comma_and_whitespace(&mut example4);
        remove_trailing_comma_and_whitespace(&mut example5);

        assert_eq!(example1, "Hello, world");
        assert_eq!(example2, "Hello, world");
        assert_eq!(example3, "Hello, world");
        assert_eq!(example4, "Hello, world");
        assert_eq!(example5, "Hello, world");
    }
}
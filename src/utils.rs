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

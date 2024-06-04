use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq)]
pub enum RunState<E: ToString> {
    #[default]
    None,
    Error(E),
    Success,
}

impl<E: ToString> RunState<E> {
    pub fn is_not_none(&self) -> bool {
        matches!(self, Self::Success | Self::Error(_))
    }

    pub fn is_none(&self) -> bool {
        !self.is_not_none()
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
}

impl<T, E: ToString> From<Result<T, E>> for RunState<E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(_) => RunState::Success,
            Err(s) => RunState::Error(s),
        }
    }
}

pub fn add_title(s: &str) -> String {
    format!(
        "################################ {} ################################\n",
        s
    )
}

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
    str1.starts_with(str2) && str2.len() >= min_length && !str2.starts_with(str1)
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

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    group_name: String,
    suffix: String,
    max: usize,
}

#[allow(unused)]
pub fn write_things(path: &str, s: impl ToString) {
    let files_path = Path::new(path);
    let settings_path = files_path.join("settings");
    let settings_file_path = settings_path.join("settings.json");
    if !settings_path.exists() {
        panic!("No settings dir");
    }

    if !settings_file_path.exists() {
        panic!("No settings json");
    }

    // 读取 settings.json 文件
    let data = fs::read_to_string(&settings_file_path).unwrap();
    let settings: Settings = serde_json::from_str(&data).unwrap();

    for i in 0..=settings.max {
        let file_path =
            files_path.join(&(settings.group_name.to_owned() + &i.to_string() + &settings.suffix));
        if !file_path.exists() {
            fs::write(file_path, s.to_string()).unwrap();
            break;
        }
    }
}

// mod test {
//     #[test]
//     fn test() {
//         super::write_things("./test_cases", "123");
//         super::write_things("./test_cases", "123");
//     }
// }

use crate::utils::RunState;
use proptest::prelude::*;
use regex::Regex;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::fs;

pub struct Tester {
    cases: Vec<(String, String)>,
    results: Vec<RunState<String>>,
}
impl Tester {
    pub fn generate_from_text(path: &str) -> Tester {
        let dir = fs::read_dir(path).expect("Invalid dir path");
        let mut cases = Vec::new();
        for file in dir {
            let file = file.unwrap();
            if file.path().is_file() {
                if let Some(ext) = file.path().extension() {
                    if ext == "txt" {
                        let contents =
                            fs::read_to_string(file.path()).expect("Can not read file contents");
                        let name = file
                            .path()
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .expect("Can not get file name");
                        cases.push((name, contents));
                    }
                }
            }
        }
        let mut results = Vec::new();
        results.resize(cases.len(), RunState::default());
        Tester { cases, results }
    }

    pub fn print_tests(&self) {
        println!("################################ TEST CASES ################################");
        for (idx, text) in self.cases.iter().enumerate() {
            println!(
                "################################ CASE{}\t: {}\t ################################",
                idx, text.0
            );
            println!("Status: {:?}", self.results[idx]);
            println!("{}", text.1);
            println!();
        }
    }

    pub fn print_res(&self) {
        println!("################################ TEST CASES ################################");
        for (idx, text) in self
            .cases
            .iter()
            .enumerate()
            .filter(|(idx, _)| self.results[*idx] != RunState::None)
        {
            println!(
                "################################ CASE{}\t: {}\t ################################",
                idx, text.0
            );
            if let RunState::Error(s) = &self.results[idx] {
                println!("Status: Error\nMessage: {}", s);
            } else if self.results[idx].is_success() {
                println!("Status: Success")
            }
            println!();
        }
    }

    pub fn is_ok(&self) -> bool {
        self.cases
            .iter()
            .enumerate()
            .filter(|(idx, _)| self.results[*idx] != RunState::None)
            .fold(true, |acc, (idx, _)| acc & !self.results[idx].is_error())
    }

    pub fn test<T>(&mut self, is_pass: T)
    where
        T: Fn(&str) -> Result<(), String>,
    {
        self.test_specific(is_pass, None)
    }

    pub fn test_specific<T>(&mut self, is_pass: T, sort_str: Option<&str>)
    where
        T: Fn(&str) -> Result<(), String>,
    {
        assert!(
            self.cases.len() == self.results.len(),
            "cases len != results len"
        );
        for (idx, (_, item)) in self
            .cases
            .iter()
            .filter(|(name, _)| {
                if let Some(sort_str) = sort_str {
                    Regex::new(sort_str)
                        .expect("Invalid Regex text")
                        .is_match(name)
                } else {
                    true
                }
            })
            .enumerate()
        {
            let res = is_pass(item);
            self.results[idx] = res.into();
        }
    }
}

#[derive(Clone, Debug)]
pub enum Json {
    Null,
    NaN,
    Infinity,
    NInfinity,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Map(HashMap<String, Json>),
}

impl Serialize for Json {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Json::Null => serializer.serialize_unit(),
            Json::NaN => serializer.serialize_f64(f64::NAN),
            Json::Infinity => serializer.serialize_f64(f64::INFINITY),
            Json::NInfinity => serializer.serialize_f64(f64::NEG_INFINITY),
            Json::Bool(b) => serializer.serialize_bool(*b),
            Json::Number(n) => serializer.serialize_f64(*n),
            Json::String(s) => serializer.serialize_str(s),
            Json::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for element in arr {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
            Json::Map(map) => {
                let mut map_serializer = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    map_serializer.serialize_entry(k, v)?;
                }
                map_serializer.end()
            }
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = json5::to_string(self).unwrap();
        write!(f, "{}", &s)
    }
}

pub fn arb_json() -> impl Strategy<Value = Json> {
    let leaf = prop_oneof![
        Just(Json::Null),
        Just(Json::NaN),
        Just(Json::Infinity),
        Just(Json::NInfinity),
        any::<bool>().prop_map(Json::Bool),
        any::<f64>().prop_filter("Must not be an integer", |&x| x.fract() != 0.0).prop_map(Json::Number),
        ".*".prop_map(Json::String),
    ];
    leaf.prop_recursive(
        8,   // 8 levels deep
        256, // Shoot for maximum size of 256 nodes
        10,  // We put up to 10 items per collection
        |inner| {
            prop_oneof![
                // Take the inner strategy and make the two recursive cases.
                prop::collection::vec(inner.clone(), 0..10).prop_map(Json::Array),
                prop::collection::hash_map(".*", inner, 0..10).prop_map(Json::Map),
            ]
        },
    )
}

mod test {
    use super::*;

    #[test]
    fn test_play() {
        let json_null = Json::Null;
        let json_bool = Json::Bool(true);
        let json_number = Json::Number(42.0);
        let json_string = Json::String("Hello, world!".to_string());
        let json_array = Json::Array(vec![Json::Number(1.0), Json::Bool(false)]);
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Json::String("value1".to_string()));
        map.insert("key2".to_string(), Json::Number(10.0));
        // let json_map = Json::Map(map);

        // println!("{}", json_null);
        // println!("{}", json_bool);
        // println!("{}", json_number);
        // println!("{}", json_string);
        // println!("{}", json_array);
        // println!("{}", json_map);

        assert!(json_null.to_string() == "null");
        assert!(json_bool.to_string() == "true");
        assert!(json_number.to_string() == "42");
        assert!(json_string.to_string() == r#""Hello, world!""#);
        assert!(json_array.to_string() == "[1,false]");
        // 这个由于map是无序的，所以确实测不了
        // assert!(json_map.to_string() == r#"{"key1":"value1","key2":10.0}"#);
    }
}

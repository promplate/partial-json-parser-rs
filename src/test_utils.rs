use crate::utils::RunState;
use regex::Regex;
use std::fs;

pub struct Tester {
    cases: Vec<(String, String)>,
    results: Vec<RunState>,
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

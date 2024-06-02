use crate::utils::{add_title, RunState};

#[derive(Default, Debug)]
pub enum State {
    InStr(EscapeCnt),
    #[default]
    NotInStr,
}

#[derive(Default, Debug)]
pub struct EscapeCnt {
    // 这是一个取值范围为[0, 2)的计数器
    cnt: usize,
    // 这是统计\u的
    u_mode: bool,
    u_cnt: usize,
}

impl EscapeCnt {
    pub fn new() -> EscapeCnt {
        EscapeCnt { cnt: 0, u_mode: false, u_cnt: 0 }
    }

    #[inline]
    fn valid_hex_char(c: &char) -> bool {
        matches!(c, 
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | 
            'a' | 'b' | 'c' | 'd' | 'e' | 'f' | 
            'A' | 'B' | 'C' | 'D' | 'E' | 'F'
        )
    }

    #[inline]
    fn valid_esc_char(&mut self, c: &char) -> bool {
        if self.cnt == 0 {
            return false;
        }
        // 实际上，在本文件中的proptest无法覆盖\u的情况
        if !self.u_mode && *c != 'u' {
            matches!(c, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't')
        } else if !self.u_mode && *c == 'u' {
            self.u_mode = true;
            false
        } else if self.u_mode && self.u_cnt < 3 && Self::valid_hex_char(c) {
            self.u_cnt += 1;
            false
        } else if self.u_mode && self.u_cnt == 3 && Self::valid_hex_char(c) {
            true
        } else {
            panic!("Should not reach here")
        }
    }

    pub fn input(&mut self, c: char) -> CharType {
        if self.cnt == 0 && c == '\\' {
            self.cnt = 1;
            CharType::Escape
        } else if self.cnt == 0 && c == '"' {
            CharType::Quotation
        } else if self.cnt == 1 && self.valid_esc_char(&c) {
            self.cnt = 0;
            self.u_cnt = 0;
            self.u_mode = false;
            CharType::Normal
        } else {
            CharType::Special
        }
    }

    pub fn cnt(&self) -> usize {
        self.cnt
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
pub enum CharType {
    Colon,     // 冒号
    Comma,     // 逗号
    Quotation, // 引号，且不代表字符'"'
    Escape,    // 转义，且不代表字符'\'
    LFB,       // left square bracket
    RFB,       // left square bracket
    LCB,       // left curly bracket
    RCB,       // right curly bracket
    #[default]
    Normal,
    Special,
}

impl CharType {
    pub fn partial_pair(&self) -> Option<CharType> {
        match self {
            Self::Quotation => Some(Self::Quotation),
            Self::LFB => Some(Self::RFB),
            Self::LCB => Some(Self::RCB),
            _ => None,
        }
    }

    pub fn partial_pair_rev(&self) -> Option<CharType> {
        match self {
            Self::Quotation => Some(Self::Quotation),
            Self::RFB => Some(Self::LFB),
            Self::RCB => Some(Self::LCB),
            _ => None,
        }
    }

    pub fn is_left_available(&self) -> bool {
        matches!(self, Self::LFB | Self::LCB)
    }

    pub fn is_right_available(&self) -> bool {
        matches!(self, Self::RFB | Self::RCB)
    }

    pub fn type_string(&self) -> String {
        let res = match self {
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Quotation => "\"",
            Self::Escape => "\\",
            Self::LFB => "[",
            Self::RFB => "]",
            Self::LCB => "{",
            Self::RCB => "}",
            Self::Normal | Self::Special => "",
        };
        res.to_string()
    }

    pub fn option_type_string(char_type: Option<CharType>) -> String {
        if let Some(t) = char_type {
            Self::type_string(&t)
        } else {
            "".to_string()
        }
    }

    pub fn simple_from_char(c: char) -> Self {
        match c {
            ':' => Self::Colon,
            ',' => Self::Comma,
            '"' => Self::Quotation,
            '\\' => Self::Escape,
            '[' => Self::LFB,
            ']' => Self::RFB,
            '{' => Self::LCB,
            '}' => Self::RCB,
            _ => Self::default(),
        }
    }
}

#[derive(Default, Debug)]
struct ParseSettings {
    // array and obj will always be cut
    allow_null: bool,
    allow_bool: bool,
    allow_number: bool,
    allow_string: bool,
    allow_infinity: bool,
    allow_ninfinity: bool,
    allow_nan: bool,
}

#[derive(Default, Debug)]
struct Parser<'a> {
    stack: Vec<(usize, CharType)>,
    state: State,
    src_str: &'a str,
    last_sep: Option<usize>,
    last_colon: Option<usize>,
    last_rbracket: Option<usize>,
    is_parsed: RunState,
    settings: ParseSettings,
}

impl<'a> Parser<'a> {
    #[allow(unused)]
    pub fn parser(in_str: &'a str) -> String {
        // 接收需要补全的字符串，返回补全后的字符串
        // 内部需要构造parser
        todo!()
    }

    pub fn stack_tracer(&self) -> String {
        let mut s = String::new();
        s.push_str(&add_title("Stack Tracer"));
        for (idx, (c_idx, item)) in self.stack.iter().enumerate() {
            let info = format!(
                "idx: {}, item {} at {} of str\n",
                idx,
                item.type_string(),
                c_idx
            );
            s.push_str(&info)
        }
        s
    }

    pub fn parse_tracer(&self) -> String {
        let mut s = String::new();
        s.push_str(&add_title("Parse Tracer"));
        s.push_str(&self.stack_tracer());
        s.push_str(&add_title("State"));
        s.push_str(&format!("{:?}\n", self.state));
        s.push_str(&add_title("Last Sep"));
        s.push_str(&format!(
            "{:?}, {}\n",
            self.last_sep,
            self.last_sep
                .map(|i| self.src_str[i..].to_string())
                .unwrap_or("None".to_string())
        ));
        if let RunState::Error(s1) = &self.is_parsed {
            s.push_str(&add_title("Parse State"));
            s.push_str(&format!("{}\n", s1));
        }
        s
    }

    pub fn parse(&mut self) {
        // 内部需要对附加的字符串src_str进行解析，并且返回修改后的结构体
        assert!(self.is_parsed.is_none());
        self.is_parsed = RunState::Success;

        for (idx, c) in self.src_str.chars().enumerate() {
            let char_type = self.state_machine_input(c);
            if char_type.is_left_available() {
                self.stack.push((idx, char_type))
            } else if char_type.is_right_available() {
                // 检查栈顶元素并对尝试进行括号闭合
                let top_item = self.stack.last().map(|(_, res)| res);
                if top_item == char_type.partial_pair_rev().as_ref() {
                    // 此时两个元素是匹配的，是正确的结果，此时应该出栈
                    self.stack.pop();
                } else {
                    // 栈顶为空或者栈顶元素不匹配，此时应该退出并报错
                    let remains = format!("remains: {}\n", &self.src_str[idx..]);
                    self.is_parsed = RunState::Error(remains);
                    println!("Warning: Stack is empty or its top element is unmatched");
                    return;
                }
            } else if char_type == CharType::Comma {
                self.last_sep = Some(idx);
            } else if char_type == CharType::Colon {
                self.last_colon = Some(idx);
            } else if char_type == CharType::RCB || char_type == CharType::RFB {
                self.last_rbracket = Some(idx)
            }
        }
    }

    // fn cut_and_amend(&self, idx: usize) -> String {
    //     // 这个函数是配合amend使用的，不能单独使用
    //     // 由amend去保证获得的是冒号后的字符切片
    //     let s = &self.src_str[idx..];
        
    // }

    // fn amend(&mut self) -> String {
    //     assert!(self.is_parsed.is_not_none());
    //     // 内部对parse后的结果进行修饰，以返回正确的结果
    //     let cut_string = if let Some(last_sep) = self.last_sep {
    //         let last_colon = self.last_colon.unwrap_or(last_sep);
    //         if last_colon <= last_sep || last_colon == self.src_str.len() - 1 {
    //             // 说明后面没有必要进行补全，直接删除即可
    //             self.src_str[..last_sep].to_string()
    //         } else {

    //         }
    //     } else {
    //         // 应该考虑没有last_sep的情况
    //         assert!(self.last_colon.is_none(), "None sep with None last colon");
    //         self.src_str.to_string()
    //     };

    // }

    fn state_machine_input(&mut self, c: char) -> CharType {
        // 先不考虑转义，字符串内部存在特殊符号的情况的情况
        match self.state {
            State::NotInStr => {
                if c == '"' {
                    self.state = State::InStr(EscapeCnt::new());
                }
                CharType::simple_from_char(c)
            }
            State::InStr(ref mut cnt) => {
                let res = cnt.input(c);
                if let CharType::Quotation = res {
                    self.state = State::NotInStr;
                }
                res
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::{arb_json, Tester};
    use proptest::prelude::*;

    use super::*;

    fn parser_full_pass(s: &str) -> Result<(), String> {
        let mut parser = Parser {
            src_str: s,
            ..Default::default()
        };
        parser.parse();
        let res = if parser.is_parsed.is_none() {
            Err("is_paresed is false\n".to_string())
        } else if parser.last_sep != s.rfind(',') {
            let res = format!("left: {:?}, right: {:?}\n", parser.last_sep, s.rfind(','));
            Err("Wrong last_sep: ".to_string() + &res + "\n")
        } else if !parser.stack.is_empty() {
            Err("Stack is not empty\n".to_string())
        } else if parser.is_parsed.is_error() {
            Err("Should not get parse error\n".to_string())
        } else {
            Ok(())
        };
        res.map_err(|err| "\n".to_string() + &err + &parser.parse_tracer())
    }

    #[test]
    fn test_full_pass() {
        let mut tester = Tester::generate_from_text("test_cases");
        tester.test_specific(parser_full_pass, Some("full[0-9]+"));
        tester.print_res();
        assert!(tester.is_ok());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn test_full_pass_prop(s in arb_json()) {
            let s = s.to_string();
            let mut parser = Parser {
                src_str: &s,
                ..Default::default()
            };
            parser.parse();
            assert!(parser.is_parsed.is_success());
            assert!(parser.stack.is_empty());
            if let State::InStr(_) = parser.state {
                panic!("State is InStr");
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(3))]
        #[test]
        fn test_my(s in arb_json()) {
            use std::io::Write;
            let s = s.to_string();
            let mut fs_ = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("./test.json").unwrap();

            writeln!(fs_, "{}", s).unwrap();
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn test_part_pass_prop(s in arb_json()) {
            let s = s.to_string();
            for (i, _) in s.char_indices() {
                if i == s.len() - 1 {
                    break;
                } else if i == 0 {
                    continue;
                }
                let mut parser = Parser {
                    src_str: &s[..i],
                    ..Default::default()
                };
                parser.parse();
                let collection_prefix = s.starts_with('[') || s.starts_with('{');
                if parser.is_parsed.is_error() || (parser.stack.is_empty() && collection_prefix) {
                    println!("String: {}", parser.src_str);
                    println!("{}, is_error: {:?}, stack: {}", parser.parse_tracer(), parser.is_parsed, parser.stack.is_empty());
                    panic!();
                }
            }
        }
    }
}

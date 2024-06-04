use crate::{
    utils::{add_title, RunState},
    value_parser,
};

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
        EscapeCnt {
            cnt: 0,
            u_mode: false,
            u_cnt: 0,
        }
    }

    #[inline]
    fn valid_hex_char(c: &char) -> bool {
        matches!(
            c,
            '0' | '1'
                | '2'
                | '3'
                | '4'
                | '5'
                | '6'
                | '7'
                | '8'
                | '9'
                | 'a'
                | 'b'
                | 'c'
                | 'd'
                | 'e'
                | 'f'
                | 'A'
                | 'B'
                | 'C'
                | 'D'
                | 'E'
                | 'F'
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

#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
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
pub struct Parser<'a> {
    stack: Vec<(usize, CharType)>,
    state: State,
    src_str: &'a str,
    last_sep: Option<usize>,
    last_colon: Option<usize>,
    last_rbracket: Option<usize>,
    is_parsed: RunState<String>,
    settings: ParseSettings,
}

impl<'a> Parser<'a> {
    #[allow(unused)]
    pub fn parser(in_str: &'a str) -> Result<String, String> {
        // 接收需要补全的字符串，返回补全后的字符串
        // 内部需要构造parser
        if in_str.is_empty() {
            return Err("Input str is Empty".to_string());
        }
        let mut parser = Parser {
            src_str: in_str,
            ..Default::default()
        };
        parser.parse();
        parser.amend().or(Err("Amend Error in parser".to_string()))
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

        for (idx, c) in self.src_str.char_indices() {
            let char_type = self.state_machine_input(c);
            if char_type.is_left_available() {
                self.stack.push((idx, char_type))
            } else if char_type.is_right_available() {
                // 检查栈顶元素并对尝试进行括号闭合
                let top_item = self.stack.last().map(|(_, res)| res);
                // 将右括号加入栈顶
                self.last_rbracket = Some(idx);
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
            }
        }
    }

    fn stack_recover(&mut self, idx: usize) {
        while let Some((top_idx, _)) = self.stack.last() {
            if *top_idx < idx {
                break;
            } else {
                self.stack.pop();
            }
        }
    }

    #[inline]
    fn cut_and_amend(&mut self, idx: usize, allow_string: bool) -> Result<String, bool> {
        // error的bool表示是否已经匹配成功，匹配成功但是不完整Err(true)，没有命中返回Err(false)

        // 获取冒号后的字符切片
        let s = &self.src_str[idx..];
        let (s, _) = value_parser::sp(s).unwrap();

        #[inline]
        // 定义一个通用的解析和校验函数
        fn parse_and_check<F>(
            _par: &mut Parser,
            _idx: usize,
            s: &str,
            parse_func: F,
            allow_incomplete: bool,
        ) -> Result<(bool, String), ()>
        where
            F: Fn(&str) -> Result<value_parser::VParserRes, ()>,
        {
            if let Ok(parse_res) = parse_func(s) {
                if allow_incomplete || parse_res.is_complete() {
                    return Ok((true, s.to_string()));
                } else {
                    return Ok((false, String::new()));
                }
            }
            Err(())
        }

        // 尝试解析bool
        parse_and_check(
            self,
            idx,
            s,
            value_parser::parse_bool,
            self.settings.allow_bool,
        )
        // 如果解析bool失败，尝试解析字符串
        .or_else(|_| {
            parse_and_check(
                self,
                idx,
                s,
                value_parser::parse_string,
                allow_string && self.settings.allow_string,
            )
        })
        .or_else(|_| {
            parse_and_check(
                self,
                idx,
                s,
                value_parser::parse_num,
                self.settings.allow_number,
            )
        })
        // 如果解析数字失败，尝试解析其它特殊字符
        .or_else(|_| {
            parse_and_check(
                self,
                idx,
                s,
                value_parser::parse_nan,
                self.settings.allow_nan,
            )
        })
        .or_else(|_| {
            parse_and_check(
                self,
                idx,
                s,
                value_parser::parse_null,
                self.settings.allow_null,
            )
        })
        .or_else(|_| {
            parse_and_check(
                self,
                idx,
                s,
                value_parser::parse_infinity,
                self.settings.allow_infinity,
            )
        })
        .or_else(|_| {
            parse_and_check(
                self,
                idx,
                s,
                value_parser::parse_ninfinity,
                self.settings.allow_ninfinity,
            )
        })
        .or(Err(false))
        .and_then(|(res, s)| if res { Ok(s) } else { Err(true) })
    }

    #[inline]
    fn get_recover_idx(&self, colon_idx: Option<usize>) -> Result<usize, ()> {
        if let Some(colon_idx) = colon_idx {
            self.stack
                .iter()
                .rev()
                .find(|(idx, _)| *idx < colon_idx)
                .map(|(idx, _)| *idx + 1)
                .ok_or(())
        } else {
            Ok(self.stack.last().unwrap().0)
        }
    }

    fn get_is_amend(&self, sep_idx: Option<usize>) -> Option<bool> {
        // 最新的，当前','之前的括号决定了这个组是obj还是arr
        // 如果这个括号过时了呢？应该找最新的符号的
        if let Some(sep_idx) = sep_idx {
            self.stack
                .iter()
                .rev()
                .find(|(idx, c)| *idx < sep_idx)
                .map(|(_, c)| *c == CharType::LCB)
        } else {
            self.stack.last().map(|(_, c)| *c == CharType::LCB)
        }
    }

    fn amend(&mut self) -> Result<String, ()> {
        assert!(self.is_parsed.is_not_none());
        if self.is_parsed.is_error() {
            return Err(());
        } else if self.is_parsed.is_success() && self.stack.is_empty() {
            match self.cut_and_amend(0, true) {
                Ok(res) => return Ok(res),
                Err(_) => {
                    if self.last_rbracket.is_some() {
                        // 说明曾经存在括号
                        return Ok(self.src_str.to_string());
                    } else {
                        return Err(());
                    }
                }
            }
        }

        let mut cur_string = String::new();
        let valid_idx: Option<i128>;
        let mut amend_system: Option<bool> = None; // false对应[, true对应{
        let recover_idx: usize; // 用于恢复的idx，仅当需要恢复时使用
        let top_elem = self.stack.last();

        let last_rbracket = self.last_rbracket;

        // 注意，目前这里存储的所有都是字节序
        if let Some(last_colon) = self.last_colon {
            if let Some(last_sep) = self.last_sep {
                valid_idx = if last_colon > last_sep {
                    Some(last_colon as i128)
                } else {
                    assert!(last_colon != last_sep);
                    amend_system = self.get_is_amend(Some(self.last_sep.unwrap()));
                    Some(last_sep as i128)
                };
                recover_idx = last_sep;
            } else {
                valid_idx = Some(last_colon as i128);
                // 此时使用top_elem来recover有可能出错，如[{"":[
                // recover_idx = top_elem.map_or(1, |(i, _)| i + 1);
                recover_idx = self.get_recover_idx(Some(last_colon))?
            }
        } else if let Some(last_sep) = self.last_sep {
            amend_system = self.get_is_amend(Some(last_sep));
            valid_idx = Some(last_sep as i128);
            recover_idx = last_sep;
        } else {
            amend_system = self.get_is_amend(None);
            // 如果是null的话，这个是对的么？
            valid_idx = top_elem.map_or(Some(-1), |(i, _)| Some(*i as i128));
            // 这时候使用top_elem作为recovery应该不会出错
            recover_idx = top_elem.map_or(0, |(i, _)| *i);
        }

        if let Some(valid_idx) = valid_idx {
            let last_rbracket = last_rbracket.map_or(valid_idx, |i| i as i128);

            // 外部需要保证len不为0
            if valid_idx == (self.src_str.len() - 1) as i128 {
                self.stack_recover(recover_idx);
                cur_string.push_str(&self.src_str[..recover_idx]);
            } else if last_rbracket <= valid_idx {
                let keyval_only = amend_system.map_or(false, |c| c);
                if !keyval_only {
                    if let Ok(s) = self.cut_and_amend((valid_idx + 1) as usize, keyval_only) {
                        cur_string.push_str(&self.src_str[..(valid_idx + 1) as usize]);
                        cur_string.push_str(&s);
                    } else {
                        // 此时cut_and_amend匹配失败，因此需要进行恢复
                        self.stack_recover(recover_idx);
                        cur_string.push_str(&self.src_str[..recover_idx]);
                    }
                } else {
                    // 此时只匹配key_val，因此需要进行恢复
                    self.stack_recover(recover_idx);
                    cur_string.push_str(&self.src_str[..recover_idx]);
                }
            } else {
                cur_string.push_str(&self.src_str[..=last_rbracket as usize]);
            }
        } else {
            return Err(());
        }

        for (_, c) in self.stack.iter().rev() {
            let s = CharType::option_type_string(c.partial_pair());
            cur_string.push_str(&s);
        }
        if cur_string.is_empty() {
            Err(())
        } else {
            Ok(cur_string)
        }
    }

    fn state_machine_input(&mut self, c: char) -> CharType {
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
    use crate::{test_utils::{arb_json, Tester}, utils};
    use proptest::prelude::*;

    use super::*;

    use serde_json::Value;

    fn is_valid_json(json_str: &str) -> bool {
        if let Err(err) = json5::from_str::<Value>(json_str) {
            println!("Error: {:?}", err);
            false
        } else {
            true
        }
    }

    // #[test]
    // fn temp() {
    //     let s = "Hello, 世界";
    //     let last_rbracket = 6; // 索引 6 是 ',' 后面的空格字符位置

    //     for (idx, c) in s.char_indices() {
    //         println!("{}, {}", idx, c)
    //     }

    //     // 这将正常工作，因为 6 是有效的 UTF-8 字符边界
    //     let slice = &s[..last_rbracket];
    //     println!("{}", slice); // 输出 "Hello,"
    // }

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
    fn parser_test_full_pass() {
        let mut tester = Tester::generate_from_text("test_cases");
        tester.test_specific(parser_full_pass, Some("full[0-9]+"));
        tester.print_res();
        assert!(tester.is_ok());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn parser_test_full_pass_prop(s in arb_json()) {
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
        #![proptest_config(ProptestConfig::with_cases(10000))]
        #[test]
        fn parser_test_part_pass_prop(s in arb_json()) {
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

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn parser_test_pass_prop(s in arb_json()) {
            let s = s.to_string();
            for (i, _) in s.char_indices() {
                if i == 0 {
                    continue;
                }
                let res = Parser::parser(&s[..i]);
                // println!("input: {}, {:?}", &s[..i], res);
                if let Ok(res) = res {
                    let json_parsed = is_valid_json(&res);
                    if !json_parsed {
                        panic!("failed_str: {:?}", &s[..i]);
                    }
                }
            }
            let res = Parser::parser(&s);
            if let Ok(res) = res {
                let json_parsed = is_valid_json(&res);
                if !json_parsed {
                    panic!("failed_str: {:?}", &s);
                }
            }
        }
    }

    // proptest! {
    //     #![proptest_config(ProptestConfig::with_cases(20))]
    //     #[test]
    //     fn performance(s in arb_json()) {
    //         utils::write_things("./test_cases", s);
    //     }
    // }

    #[test]
    fn amend_test_part_pass() {
        let list = [
            r#"[{"*\t<򀣺󼚨  $񺆨=.?'\/\/ 򎎨􂊖`":true}]"#,
            r#"[["¡¡¡¡",null]]"#,
            r#"[{"M\t     ":"|","*\t<򀣺󼚨  $񺆨=.?'\/\/ 򎎨􂊖`":true}]"#,
            r#"null"#,
            r#"{"":null,"󮐋":NaN}"#,
            r#"[[null,[]]]"#,
            r#"[{"*\t<򀣺󼚨  $񺆨=.?'\/\/ 򎎨􂊖`":true,"":0}]"#,
            r#"[[null,[null]]]"#,
            r#"[null,{}]"#,
            r#"[{"":[]}]"#,
            r#"[null,{"":null}]"#,
            r#"[{"L󼸑":[-Infinity],"G𒇗\/:O=":false}]"#,
            r#"[null]"#,
            "{\"\":{\"\":{",
            r#"["a", [[12, []]"#,
            r#"["a", [[12, {"": { "": {}, "": {}"#,
            r#"-Infinity"#,
        ];
        for s in list {
            // println!("{}", s);
            for (i, _) in s.char_indices() {
                if i == 0 {
                    continue;
                }
                let mut parser = Parser {
                    src_str: &s[..i],
                    ..Default::default()
                };
                parser.parse();
                let res = parser.amend();
                println!("input: {}, {:?}", &s[..i], res);
                if let Ok(res) = res {
                    assert!(is_valid_json(&res));
                }
            }
        }
    }

    #[test]
    fn amend_test_full_pass() {
        let list = [
            r#"[{"*\t<򀣺󼚨  $񺆨=.?'\/\/ 򎎨􂊖`":true}]"#,
            r#"[["¡¡¡¡",null]]"#,
            r#"[{"M\t     ":"|","*\t<򀣺󼚨  $񺆨=.?'\/\/ 򎎨􂊖`":true}]"#,
            r#"null"#,
            r#"{"":null,"󮐋":NaN}"#,
            r#"[[null,[]]]"#,
            r#"[{"*\t<򀣺󼚨  $񺆨=.?'\/\/ 򎎨􂊖`":true,"":0}]"#,
            r#"[[null,[null]]]"#,
            r#"[null,{}]"#,
            r#"[{"":[]}]"#,
            r#"[null,{"":null}]"#,
            r#"[{"L󼸑":[-Infinity],"G𒇗\/:O=":false}]"#,
            r#"[null]"#,
            r#""-Infinity""#,
        ];
        for s in list {
            if !is_valid_json(s) {
                continue;
            }
            println!("{}", s);
            let mut parser = Parser {
                src_str: s,
                ..Default::default()
            };
            parser.parse();
            let res = parser.amend();
            println!("input: {}, {:?}", s, res);
            if let Ok(res) = res {
                assert!(is_valid_json(&res));
            } else {
                panic!("Should Ok")
            }
            assert!(is_valid_json(s));
        }
    }
}

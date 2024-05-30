use crate::utils::{add_title, RunState};

#[derive(Default, Debug)]
enum State {
    InStr,
    #[default]
    NotInStr,
}

#[derive(Default, PartialEq, Eq, Debug)]
enum CharType {
    Colon, // 冒号
    Comma, // 逗号
    Quotation,
    LFB, // left square bracket
    RFB, // left square bracket
    LCB, // left curly bracket
    RCB, // right curly bracket
    #[default]
    Normal,
}

impl CharType {
    fn partial_pair(&self) -> Option<CharType> {
        match self {
            Self::Quotation => Some(Self::Quotation),
            Self::LFB => Some(Self::RFB),
            Self::LCB => Some(Self::RCB),
            _ => None,
        }
    }

    fn partial_pair_rev(&self) -> Option<CharType> {
        match self {
            Self::Quotation => Some(Self::Quotation),
            Self::RFB => Some(Self::LFB),
            Self::RCB => Some(Self::LCB),
            _ => None,
        }
    }

    fn is_left_available(&self) -> bool {
        matches!(self, Self::LFB | Self::LCB)
    }

    fn is_right_available(&self) -> bool {
        matches!(self, Self::RFB | Self::RCB)
    }

    fn type_string(&self) -> String {
        let res = match self {
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Quotation => "\"",
            Self::LFB => "[",
            Self::RFB => "]",
            Self::LCB => "{",
            Self::RCB => "}",
            Self::Normal => "",
        };
        res.to_string()
    }

    fn option_type_string(char_type: Option<CharType>) -> String {
        if let Some(t) = char_type {
            Self::type_string(&t)
        } else {
            "".to_string()
        }
    }

    fn simple_from_char(c: char) -> Self {
        match c {
            ':' => Self::Colon,
            ',' => Self::Comma,
            '\"' => Self::Quotation,
            '[' => Self::LFB,
            ']' => Self::RFB,
            '{' => Self::LCB,
            '}' => Self::RCB,
            _ => Self::default(),
        }
    }
}

#[derive(Default, Debug)]
struct Parser<'a> {
    stack: Vec<(usize, CharType)>,
    state: State,
    src_str: &'a str,
    last_sep: Option<usize>,
    is_parsed: RunState,
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
        s.push_str(&format!("{:?}\n", self.last_sep));
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
            }
        }
    }

    #[allow(unused)]
    fn amend(&mut self) {
        assert!(self.is_parsed.is_not_none());
        // 内部对parse后的结果进行修饰，以返回正确的结果
    }

    fn state_machine_input(&mut self, c: char) -> CharType {
        // 先不考虑转义，字符串内部存在特殊符号的情况的情况
        CharType::simple_from_char(c)
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::Tester;

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
}

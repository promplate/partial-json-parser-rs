
#[derive(Default)]
enum State {
    InStr,
    #[default]
    NotInStr,
}

#[derive(Default, PartialEq, Eq)]
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
        matches!(self, Self::Quotation | Self::LFB | Self::LCB)
    }

    fn is_right_available(&self) -> bool {
        matches!(self, Self::Quotation | Self::RFB | Self::RCB)
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
}

#[derive(Default)]
struct Parser<'a> {
    stack: Vec<(usize, CharType)>,
    state: State,
    src_str: &'a str,
    last_sep: Option<usize>,
    is_parsed: bool,
    parse_err: bool,
}

impl<'a> Parser<'a> {
    #[allow(unused)]
    pub fn parser(in_str: &'a str) -> String {
        // 接收需要补全的字符串，返回补全后的字符串
        // 内部需要构造parser
        todo!()
    }

    pub fn parse(&mut self) {
        // 内部需要对附加的字符串src_str进行解析，并且返回修改后的结构体
        assert!(!self.is_parsed);
        self.is_parsed = true;
        self.parse_err = false;

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
                    self.parse_err = true;
                    println!("Warning: Stack is empty or its top element is unmatched");
                    return;
                }
            }
        }

    }

    #[allow(unused)]
    fn amend(&mut self) {
        assert!(self.is_parsed);
        // 内部对parse后的结果进行修饰，以返回正确的结果
    }

    fn state_machine_input(&mut self, c: char) -> CharType {
        // 先不考虑转义，字符串内部存在特殊符号的情况的情况
        match c {
            ':' => CharType::Colon,
            ',' => CharType::Comma,
            '\"' => CharType::Quotation,
            _ => CharType::default(),
        }
    }
}

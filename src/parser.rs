use std::default;


enum State {
    InStr,
    NotInStr,
}

#[derive(Default)]
enum CharType {
    Colon,
    Comma,
    Quotation,
    LFB, // left square bracket
    RFB, // left square bracket
    LCB, // left curly bracket
    RCB, // right curly bracket
    #[default]
    Normal,
}

struct Parser<'a> {
    stack: Vec<(usize, CharType)>,
    state: State,
    src_str: &'a str,
    last_sep: Option<usize>,
    is_parsed: bool,
}

impl <'a> Parser<'a> {
    #[allow(unused)]
    fn parser(in_str: &'a str) -> String {
        // 接收需要补全的字符串，返回补全后的字符串
        // 内部需要构造parser
        todo!()
    }

    fn parse(&mut self) {
        // 内部需要对附加的字符串src_str进行解析，并且返回修改后的结构体
        assert!(!self.is_parsed);
    
        for (idx, c) in self.src_str.chars().enumerate() {
            let char_type = self.state_machine_input(c);
        }

        self.is_parsed = true;
    }

    #[allow(unused)]
    fn amend(&mut self) {
        assert!(self.is_parsed);
        // 内部对parse后的结果进行修饰，以返回正确的结果
    }

    fn state_machine_input(&mut self, c: char) -> CharType {
        // 先不考虑转义，字符串内部存在特殊符号的情况的情况
        match c {
            ':'=> CharType::Colon,
            ','=> CharType::Comma,
            '\"'=> CharType::Quotation,
            _ => CharType::default(),
        }
    }
}
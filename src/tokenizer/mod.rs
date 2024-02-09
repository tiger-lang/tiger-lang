use std::{
    collections::VecDeque,
    io::{self, Read},
};

#[derive(Debug, Clone)]
pub struct Token {
    pub column: usize,
    pub line: usize,
    pub length: usize,
    pub path: String,
    pub text: String,
    pub value: TokenValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Increment,
    Decrement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    BinaryOr,
    BinaryAnd,
    Xor,
    LogicalOr,
    LogicalAnd,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equals,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
    NotEquals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignOperator {
    Assign,
    AssignAfter(BinaryOperator),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Identifier(String),

    IntegerLiteral(i128),
    FloatingPointLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    UnaryOperator(UnaryOperator),
    BinaryOperator(BinaryOperator),
    Comparison(ComparisonOperator),
    Assignment(AssignOperator),

    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Dot,
    Comma,
    Newline,

    KeywordFunc,
    KeywordTest,
    KeywordIf,
    KeywordElse,
    KeywordFor,
    KeywordLoop,
    KeywordWhile,
    KeywordVar,
    KeywordConst,
    KeywordUse,
    KeywordAs,
    KeywordReturn,
}

pub enum ErrorKind {
    InvalidInput,
    Internal,
    IOError(io::Error),
}

pub struct Error {
    pub message: String,
    pub kind: ErrorKind,

    pub line: usize,
    pub column: usize,
    pub source: String,
}

/// TokenStream provides an easy way to iterate over the tokenized contents of some tiger source input.
pub struct TokenStream<R: Read> {
    stream_path: String,
    stream: R,
    stream_column: usize,
    stream_line: usize,
    lookahead_buf: VecDeque<char>,
    finished: bool,
    cached_token: Option<Token>,

    token_column: usize,
    token_line: usize,
}

impl<R: Read> TokenStream<R> {
    /// Create a new TokenStream from a reader and an optional source path
    pub fn new(r: R, path: Option<String>) -> Self {
        let path = if let Some(p) = path {
            p
        } else {
            "-".to_string()
        };
        return Self {
            stream_path: path,
            stream: r,
            stream_column: 1,
            stream_line: 1,
            token_column: 1,
            token_line: 1,
            lookahead_buf: VecDeque::new(),
            finished: false,
            cached_token: None,
        };
    }

    pub fn is_empty(&self) -> bool {
        self.finished && self.cached_token.is_none()
    }

    fn next_char(&mut self) -> Option<Result<char, io::Error>> {
        let res: Option<Result<char, io::Error>>;

        if !self.lookahead_buf.is_empty() {
            res = self.lookahead_buf.pop_front().map(|v| Ok(v));
        } else {
            res = read_char(&mut self.stream);
        }

        match res {
            Some(Ok('\n')) => {
                self.stream_column = 1;
                self.stream_line += 1;
            }
            Some(Ok(_)) => {
                self.stream_column += 1;
            }
            _ => (),
        }

        if res.is_none() {
            self.finished = true;
        }

        return res;
    }

    pub fn peek(&mut self) -> Option<Result<Token, Error>> {
        let res = self.next_token();

        match &res {
            Some(Ok(t)) => {
                self.cached_token = Some(t.clone());
            }
            _ => (),
        }

        res
    }

    /// Reads the next token from the stream.
    ///
    /// Returns None when EOF has been reached without errors.
    pub fn next_token(&mut self) -> Option<Result<Token, Error>> {
        if self.cached_token.is_some() {
            let res = self.cached_token.take().map(|t| Ok(t));
            return res;
        }

        loop {
            self.token_column = self.stream_column;
            self.token_line = self.stream_line;

            let v = match self.next_char() {
                None => return None, // EOF
                Some(Ok('\n')) => self.build_token(TokenValue::Newline, "\n"),
                Some(Ok(c)) if c.is_whitespace() => continue,
                Some(Ok(c)) if c == '_' || c.is_alphabetic() => {
                    self.push_char(c);
                    return self.read_ident();
                }
                Some(Ok(c)) if c.is_numeric() => {
                    self.push_char(c);
                    return self.read_number();
                }
                Some(Ok('{')) => self.build_token(TokenValue::OpenBrace, "{"),
                Some(Ok('}')) => self.build_token(TokenValue::CloseBrace, "}"),
                Some(Ok('[')) => self.build_token(TokenValue::OpenBracket, "["),
                Some(Ok(']')) => self.build_token(TokenValue::CloseBracket, "]"),
                Some(Ok('(')) => self.build_token(TokenValue::OpenParen, "("),
                Some(Ok(')')) => self.build_token(TokenValue::CloseParen, ")"),
                Some(Ok(',')) => self.build_token(TokenValue::Comma, ","),
                Some(Ok('.')) => self.build_token(TokenValue::Dot, "."),
                Some(Ok('"')) => return self.read_string(),
                Some(Ok('\'')) => return self.read_char(),
                Some(Ok(c)) if is_operator(c) => {
                    self.push_char(c);
                    return self.read_operator();
                }
                Some(Ok(c)) => {
                    return Some(Err(Error {
                        message: format!("unexpected character '{}'", c),
                        source: self.stream_path.clone(),
                        column: self.stream_column,
                        line: self.stream_line,
                        kind: ErrorKind::InvalidInput,
                    }))
                }
                Some(Err(err)) => return Some(Err(self.io_error(err))),
            };

            return Some(Ok(v));
        }
    }

    fn build_token(&self, value: TokenValue, text: &str) -> Token {
        Token {
            column: self.token_column,
            line: self.token_line,
            path: self.stream_path.clone(),
            length: text.len(),
            text: String::from(text),
            value,
        }
    }

    fn read_ident(&mut self) -> Option<Result<Token, Error>> {
        let mut value = Vec::new();

        loop {
            match self.next_char() {
                None => break,
                Some(Err(e)) => return Some(Err(self.io_error(e))),
                Some(Ok(c)) => {
                    if c.is_alphanumeric() || c == '_' {
                        value.push(c);
                    } else {
                        break;
                    }
                }
            }
        }

        let s: String = value.iter().collect();

        if let Some(t) = self.check_keyword(s.as_str()) {
            return Some(Ok(t));
        }

        Some(Ok(
            self.build_token(TokenValue::Identifier(s.clone()), s.as_str())
        ))
    }

    fn check_keyword(&self, s: &str) -> Option<Token> {
        match s {
            "if" => Some(self.build_token(TokenValue::KeywordIf, s)),
            "else" => Some(self.build_token(TokenValue::KeywordElse, s)),
            "for" => Some(self.build_token(TokenValue::KeywordFor, s)),
            "loop" => Some(self.build_token(TokenValue::KeywordLoop, s)),
            "while" => Some(self.build_token(TokenValue::KeywordWhile, s)),
            "func" => Some(self.build_token(TokenValue::KeywordFunc, s)),
            "test" => Some(self.build_token(TokenValue::KeywordTest, s)),
            "var" => Some(self.build_token(TokenValue::KeywordVar, s)),
            "const" => Some(self.build_token(TokenValue::KeywordConst, s)),
            "use" => Some(self.build_token(TokenValue::KeywordUse, s)),
            "as" => Some(self.build_token(TokenValue::KeywordAs, s)),
            "return" => Some(self.build_token(TokenValue::KeywordReturn, s)),
            "true" => Some(self.build_token(TokenValue::BoolLiteral(true), s)),
            "false" => Some(self.build_token(TokenValue::BoolLiteral(false), s)),
            _ => None,
        }
    }

    fn read_number(&mut self) -> Option<Result<Token, Error>> {
        let mut base = 10;
        let mut floating_point = false;
        let mut value = Vec::new();

        loop {
            match self.next_char() {
                None => break,
                Some(Ok(c)) => {
                    match c {
                        '-' if value.is_empty() => {
                            value.push(c);
                        }
                        '.' if value.is_empty() => {
                            if base != 10 || floating_point {
                                return Some(Err(self.error(
                                    "unexpected character '.' in a number literal".to_string(),
                                )));
                            }
                            floating_point = true;
                        }
                        'x' if value.is_empty() => base = 16,
                        'b' if value.is_empty() => base = 2,
                        'o' if value.is_empty() => base = 8,
                        c if is_numeric(c, base) => value.push(c),
                        _ => {
                            if c.is_alphanumeric() || c == '_' {
                                return Some(Err(self.error(format!(
                                    "unexpected character '{}' in a number literal",
                                    c
                                ))));
                            }
                            // Non-numeric character that's not alphanumeric - assumed to be the start of the next token
                            self.push_char(c);
                            break;
                        }
                    }
                }
                Some(Err(e)) => return Some(Err(self.io_error(e))),
            }
        }

        if value.last().is_some_and(|c| *c == '.') {
            // A char at the end of a number is interpreted as a dot token, not part of a floating point number
            value.pop();
            self.push_char('.');
            floating_point = false;
        }

        let s: String = value.iter().collect();
        if floating_point {
            match s.parse::<f64>() {
                Ok(fp) => {
                    return Some(Ok(
                        self.build_token(TokenValue::FloatingPointLiteral(fp), s.as_str())
                    ))
                }
                Err(e) => {
                    return Some(Err(self.internal_error(format!(
                        "internal error while reading number: {}",
                        e.to_string()
                    ))))
                }
            }
        }

        let value_string: String = value.into_iter().collect();
        match i128::from_str_radix(&value_string, base) {
            Ok(i) => {
                return Some(Ok(
                    self.build_token(TokenValue::IntegerLiteral(i), s.as_str())
                ))
            }
            Err(e) => {
                return Some(Err(self.internal_error(format!(
                    "internal error while reading number: {}",
                    e.to_string()
                ))))
            }
        }
    }

    fn read_string(&mut self) -> Option<Result<Token, Error>> {
        let mut buf = Vec::new();

        loop {
            match self.next_char() {
                Some(Ok(c)) => match c {
                    '"' => break,
                    '\\' => match self.read_escape_sequence() {
                        Ok(c) => buf.push(c),
                        Err(e) => return Some(Err(e)),
                    },
                    _ => buf.push(c),
                },
                None => {
                    return Some(Err(self.error(
                        "unexpected EOF while reading unterminated string".to_string(),
                    )))
                }
                Some(Err(e)) => return Some(Err(self.io_error(e))),
            }
        }

        let s: String = buf.iter().collect();
        Some(Ok(self.build_token(
            TokenValue::StringLiteral(s.clone()),
            format!("\"{}\"", s).as_str(),
        )))
    }

    fn read_char(&mut self) -> Option<Result<Token, Error>> {
        let res = match self.next_char() {
            Some(Ok('\'')) => {
                return Some(Err(
                    self.error("character literal cannot be empty".to_string())
                ))
            }
            Some(Ok('\\')) => match self.read_escape_sequence() {
                Ok(c) => c,
                Err(e) => return Some(Err(e)),
            },
            Some(Ok(c)) => c,
            None => {
                return Some(Err(self.error(
                    "unexpected EOF while reading character literal".to_string(),
                )))
            }
            Some(Err(e)) => return Some(Err(self.io_error(e))),
        };

        match self.next_char() {
            Some(Ok('\'')) => (),
            Some(Ok(c)) => {
                return Some(Err(self.error(format!(
                    "unexpected character '{}': character literals can only contain one character",
                    c
                ))))
            }
            None => {
                return Some(Err(self.error(
                    "unexpected EOF while reading unterminated string".to_string(),
                )))
            }
            Some(Err(e)) => return Some(Err(self.io_error(e))),
        };

        Some(Ok(self.build_token(
            TokenValue::CharLiteral(res),
            format!("'{}'", res).as_str(),
        )))
    }

    fn read_operator(&mut self) -> Option<Result<Token, Error>> {
        match self.next_char() {
            None => None,
            Some(Ok(c1)) => {
                match c1 {
                    '=' | '*' | '/' | '%' | '^' | '!' => {
                        // can be c or c=
                        let c2 = match self.next_char() {
                            Some(Ok(v)) => Some(v),
                            Some(Err(e)) => return Some(Err(self.io_error(e))),
                            None => None,
                        };

                        if let Some(c2) = c2 {
                            if c2 == '=' {
                                self.build_operator(format!("{}{}", c1, c2).as_str())
                            } else {
                                self.push_char(c2);
                                self.build_operator(format!("{}", c1).as_str())
                            }
                        } else {
                            self.build_operator(format!("{}", c1).as_str())
                        }
                    }
                    '&' | '|' | '<' | '>' => {
                        // can be c, cc, c= or cc=
                        let mut res = format!("{}", c1);
                        loop {
                            let c2 = match self.next_char() {
                                Some(Ok(v)) => Some(v),
                                Some(Err(e)) => return Some(Err(self.io_error(e))),
                                None => None,
                            };

                            if let Some(c2) = c2 {
                                match c2 {
                                    _ if c1 == c2 && res.len() == 1 => {
                                        res.push(c2);
                                        continue;
                                    }
                                    '=' => {
                                        res.push(c2);
                                        break;
                                    }
                                    _ => {
                                        self.push_char(c2);
                                        break;
                                    }
                                }
                            }
                            break;
                        }
                        self.build_operator(res.as_str())
                    }
                    '+' | '-' => {
                        // can be c, cc or c=
                        let c2 = match self.next_char() {
                            Some(Ok(v)) => Some(v),
                            Some(Err(e)) => return Some(Err(self.io_error(e))),
                            None => None,
                        };

                        match c2 {
                            Some(c2) if c2 == c1 || c2 == '=' => {
                                self.build_operator(format!("{}{}", c1, c2).as_str())
                            }
                            Some(_) => {
                                self.push_char(c2.unwrap());
                                self.build_operator(format!("{}", c1).as_str())
                            }
                            None => self.build_operator(format!("{}", c1).as_str()),
                        }
                    }
                    _ => Some(Err(self.error(format!(
                        "unexpected character '{}' while reading operator token",
                        c1
                    )))),
                }
            }
            Some(Err(e)) => Some(Err(self.io_error(e))),
        }
    }

    fn build_operator(&mut self, op: &str) -> Option<Result<Token, Error>> {
        match op {
            "+" => Some(Ok(
                self.build_token(TokenValue::BinaryOperator(BinaryOperator::Add), op)
            )),
            "-" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::Subtract),
                op,
            ))),
            "*" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::Multiply),
                op,
            ))),
            "/" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::Divide),
                op,
            ))),
            "%" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::Modulo),
                op,
            ))),
            "|" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::BinaryOr),
                op,
            ))),
            "&" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::BinaryAnd),
                op,
            ))),
            "^" => Some(Ok(
                self.build_token(TokenValue::BinaryOperator(BinaryOperator::Xor), op)
            )),
            "||" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::LogicalOr),
                op,
            ))),
            "&&" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::LogicalAnd),
                op,
            ))),
            "<<" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::ShiftLeft),
                op,
            ))),
            ">>" => Some(Ok(self.build_token(
                TokenValue::BinaryOperator(BinaryOperator::ShiftRight),
                op,
            ))),
            "==" => Some(Ok(
                self.build_token(TokenValue::Comparison(ComparisonOperator::Equals), op)
            )),
            ">" => Some(Ok(self.build_token(
                TokenValue::Comparison(ComparisonOperator::GreaterThan),
                op,
            ))),
            "<" => Some(Ok(self.build_token(
                TokenValue::Comparison(ComparisonOperator::LessThan),
                op,
            ))),
            ">=" => Some(Ok(self.build_token(
                TokenValue::Comparison(ComparisonOperator::GreaterThanOrEquals),
                op,
            ))),
            "<=" => Some(Ok(self.build_token(
                TokenValue::Comparison(ComparisonOperator::LessThanOrEquals),
                op,
            ))),
            "!=" => Some(Ok(self.build_token(
                TokenValue::Comparison(ComparisonOperator::NotEquals),
                op,
            ))),
            "!" => Some(Ok(
                self.build_token(TokenValue::UnaryOperator(UnaryOperator::Not), op)
            )),
            "++" => Some(Ok(self.build_token(
                TokenValue::UnaryOperator(UnaryOperator::Increment),
                op,
            ))),
            "--" => Some(Ok(self.build_token(
                TokenValue::UnaryOperator(UnaryOperator::Decrement),
                op,
            ))),
            "=" => Some(Ok(
                self.build_token(TokenValue::Assignment(AssignOperator::Assign), op)
            )),
            "+=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Add)),
                op,
            ))),
            "-=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Subtract)),
                op,
            ))),
            "*=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Multiply)),
                op,
            ))),
            "/=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Divide)),
                op,
            ))),
            "%=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Modulo)),
                op,
            ))),
            "|=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::BinaryOr)),
                op,
            ))),
            "&=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::BinaryAnd)),
                op,
            ))),
            "^=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Xor)),
                op,
            ))),
            "||=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::LogicalOr)),
                op,
            ))),
            "&&=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::LogicalAnd)),
                op,
            ))),
            "<<=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::ShiftLeft)),
                op,
            ))),
            ">>=" => Some(Ok(self.build_token(
                TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::ShiftRight)),
                op,
            ))),
            _ => Some(Err(
                self.internal_error(format!("unknown operator string {}", op))
            )),
        }
    }

    fn read_escape_sequence(&mut self) -> Result<char, Error> {
        match self.next_char() {
            Some(Ok(c)) => match c {
                'r' => return Ok('\n'),
                'n' => return Ok('\n'),
                't' => return Ok('\t'),
                '\\' | '\'' | '"' => return Ok(c),
                // TODO: unicode and hex escape codes
                _ => return Err(self.error(format!("invalid character in escape sequence: {}", c))),
            },
            None => return Err(self.error("EOF reached while reading escape sequence".to_string())),
            Some(Err(e)) => return Err(self.io_error(e)),
        }
    }

    fn push_char(&mut self, c: char) {
        self.lookahead_buf.push_back(c)
    }

    fn internal_error(&self, msg: String) -> Error {
        return Error {
            message: msg,
            line: self.stream_line,
            column: self.stream_column,
            source: self.stream_path.clone(),
            kind: ErrorKind::Internal,
        };
    }

    fn error(&self, msg: String) -> Error {
        Error {
            message: msg,
            line: self.stream_line,
            column: self.stream_column,
            source: self.stream_path.clone(),
            kind: ErrorKind::InvalidInput,
        }
    }

    fn io_error(&self, err: io::Error) -> Error {
        Error {
            message: "I/O error".into(),
            line: self.stream_line,
            column: self.stream_column,
            source: self.stream_path.clone(),
            kind: ErrorKind::IOError(err),
        }
    }

    pub fn position(&self) -> (String, usize, usize) {
        (
            self.stream_path.clone(),
            self.stream_line,
            self.stream_column,
        )
    }
}

fn read_char<R: Read>(r: &mut R) -> Option<io::Result<char>> {
    let mut rune_len = 0;
    let mut rune = [0u8; 4];
    while rune_len < 4 {
        match r.read(&mut rune[rune_len..rune_len + 1]) {
            Ok(0) => {
                if rune_len == 0 {
                    return None;
                } else {
                    return Some(Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "incomplete UTF-8 codepoint",
                    )));
                }
            }
            Ok(_) => {
                rune_len = rune_len + 1;
            }
            Err(e) => return Some(Err(e)),
        };

        match std::str::from_utf8(&rune) {
            Ok(s) => return Some(Ok(s.chars().next().unwrap())),
            Err(_) => (),
        }
    }
    return Some(Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "failed to parse UTF-8 data: {:x}{:x}{:x}{:x}",
            rune[0], rune[1], rune[2], rune[3]
        ),
    )));
}

fn is_numeric(c: char, base: u32) -> bool {
    c.to_digit(base).is_some()
}

fn is_operator(c: char) -> bool {
    "+-/*%!&|^=<>".contains(c)
}

impl<R: Read> Iterator for TokenStream<R> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod test;

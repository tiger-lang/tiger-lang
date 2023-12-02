use std::{
    collections::VecDeque,
    io::{self, Read},
};

pub struct Token {
    pub column: usize,
    pub line: usize,
    pub length: usize,
    pub text: String,
    pub value: TokenValue,
}

pub enum UnaryOperator {
    Not,
    Increment,
    Decrement,
}

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
    ShiftRight
}

pub enum ComparisonOperator {
    Equals,
    GreaterThan,
    LessThan,
    GreaterThanOrEquals,
    LessThanOrEquals,
    NotEquals,
}

pub enum AssignOperator {
    Assign,
    AssignAfter(BinaryOperator),
}

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
}

pub enum ErrorKind {
    InvalidInput,
    Internal,
    IOError(io::Error),
}

pub struct Error {
    pub message: String,
    pub kind: ErrorKind,
}

impl Error {}

/// TokenStream provides an easy way to iterate over the tokenized contents of some tiger source input.
pub struct TokenStream<R: Read> {
    stream_path: String,
    stream: R,
    stream_column: usize,
    stream_line: usize,
    lookahead_buf: VecDeque<char>,

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
        };
    }

    fn next_char(&mut self) -> Result<Option<char>, io::Error> {
        let res: Result<Option<char>, io::Error>;

        if !self.lookahead_buf.is_empty() {
            res = Ok(self.lookahead_buf.pop_front());
        } else {
            res = read_char(&mut self.stream);
        }

        match res {
            Ok(Some('\n')) => {
                self.stream_column = 1;
                self.stream_line += 1;
            }
            Ok(Some(_)) => {
                self.stream_column += 1;
            }
            _ => (),
        }

        return res;
    }

    /// Reads the next token from the stream.
    ///
    /// Returns Ok(None) when EOF has been reached without errors.
    pub fn next(&mut self) -> Result<Option<Token>, Error> {
        loop {
            self.token_column = self.stream_column;
            self.token_line = self.stream_line;

            let v = match self.next_char() {
                Ok(None) => return Ok(None), // EOF
                Ok(Some(c)) if c.is_whitespace() => continue,
                Ok(Some(c)) if c == '_' || c.is_alphabetic() => {
                    self.push_char(c);
                    return self.read_ident();
                }
                Ok(Some(c)) if c.is_numeric() => {
                    self.push_char(c);
                    return self.read_number();
                }
                Ok(Some('{')) => self.build_token(TokenValue::OpenBrace, "{"),
                Ok(Some('}')) => self.build_token(TokenValue::CloseBrace, "}"),
                Ok(Some('[')) => self.build_token(TokenValue::OpenBracket, "["),
                Ok(Some(']')) => self.build_token(TokenValue::CloseBracket, "]"),
                Ok(Some('(')) => self.build_token(TokenValue::OpenParen, "("),
                Ok(Some(')')) => self.build_token(TokenValue::CloseParen, ")"),
                Ok(Some(',')) => self.build_token(TokenValue::Comma, ","),
                Ok(Some('.')) => self.build_token(TokenValue::Dot, "."),
                Ok(Some('"')) => return self.read_string(),
                Ok(Some('\'')) => return self.read_char(),
                Ok(Some(c)) if is_operator(c) => {
                    self.push_char(c);
                    return self.read_operator();
                }
                Ok(Some(c)) => {
                    return Err(Error {
                        message: format!(
                            "unexpected character '{}' at line {} column {}",
                            c, self.stream_line, self.stream_column
                        ),
                        kind: ErrorKind::InvalidInput,
                    })
                }
                Err(err) => return self.io_error(err),
            };

            return Ok(Some(v));
        }
    }

    fn build_token(&self, value: TokenValue, text: &str) -> Token {
        Token {
            column: self.token_column,
            line: self.token_line,
            length: text.len(),
            text: String::from(text),
            value,
        }
    }

    fn read_ident(&mut self) -> Result<Option<Token>, Error> {
        let mut value = Vec::new();

        loop {
            match self.next_char() {
                Ok(None) => break,
                Err(e) => return self.io_error(e),
                Ok(Some(c)) => {
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
            return Ok(Some(t));
        }

        Ok(Some(self.build_token(
            TokenValue::Identifier(s.clone()),
            s.as_str(),
        )))
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
            "true" => Some(self.build_token(TokenValue::BoolLiteral(true), s)),
            "false" => Some(self.build_token(TokenValue::BoolLiteral(false), s)),
            _ => None,
        }
    }

    fn read_number(&mut self) -> Result<Option<Token>, Error> {
        let mut base = 10;
        let mut floating_point = false;
        let mut value = Vec::new();

        loop {
            match self.next_char() {
                Ok(None) => break,
                Ok(Some(c)) => {
                    match c {
                        '-' if value.is_empty() => {
                            value.push(c);
                        }
                        '.' if value.is_empty() => {
                            if base != 10 || floating_point {
                                return self.error(
                                    "unexpected character '.' in a number literal".to_string(),
                                );
                            }
                            floating_point = true;
                        }
                        'x' if value.is_empty() => base = 16,
                        'b' if value.is_empty() => base = 2,
                        'o' if value.is_empty() => base = 8,
                        c if is_numeric(c, base) => value.push(c),
                        _ => {
                            if c.is_alphanumeric() || c == '_' {
                                return self.error(format!(
                                    "unexpected character '{}' in a number literal",
                                    c
                                ));
                            }
                            // Non-numeric character that's not alphanumeric - assumed to be the start of the next token
                            self.push_char(c);
                            break;
                        }
                    }
                }
                Err(e) => return self.io_error(e),
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
                    return Ok(Some(
                        self.build_token(TokenValue::FloatingPointLiteral(fp), s.as_str()),
                    ))
                }
                Err(e) => {
                    return self.internal_error(format!(
                        "internal error while reading number: {}",
                        e.to_string()
                    ))
                }
            }
        }

        let value_string: String = value.into_iter().collect();
        match i128::from_str_radix(&value_string, base) {
            Ok(i) => {
                return Ok(Some(
                    self.build_token(TokenValue::IntegerLiteral(i), s.as_str()),
                ))
            }
            Err(e) => {
                return self.internal_error(format!(
                    "internal error while reading number: {}",
                    e.to_string()
                ))
            }
        }
    }

    fn read_string(&mut self) -> Result<Option<Token>, Error> {
        let mut buf = Vec::new();

        loop {
            match self.next_char() {
                Ok(c) => {
                    if let Some(c) = c {
                        match c {
                            '"' => break,
                            '\\' => match self.read_escape_sequence() {
                                Ok(c) => buf.push(c),
                                Err(e) => return Err(e),
                            },
                            _ => buf.push(c),
                        }
                    } else {
                        return self
                            .error("unexpected EOF while reading unterminated string".to_string());
                    }
                }
                Err(e) => return self.io_error(e),
            }
        }

        let s: String = buf.iter().collect();
        Ok(Some(self.build_token(
            TokenValue::StringLiteral(s.clone()),
            format!("\"{}\"", s).as_str(),
        )))
    }

    fn read_char(&mut self) -> Result<Option<Token>, Error> {
        let res = match self.next_char() {
            Ok(Some('\'')) => return self.error("character literal cannot be empty".to_string()),
            Ok(Some('\\')) => match self.read_escape_sequence() {
                Ok(c) => c,
                Err(e) => return Err(e),
            },
            Ok(Some(c)) => c,
            Ok(None) => {
                return self.error("unexpected EOF while reading character literal".to_string())
            }
            Err(e) => return self.io_error(e),
        };

        match self.next_char() {
            Ok(Some('\'')) => (),
            Ok(Some(c)) => {
                return self.error(
                    "unexpected character '{}': character literals can only contain one character"
                        .to_string(),
                )
            }
            Ok(None) => {
                return self.error("unexpected EOF while reading unterminated string".to_string())
            }
            Err(e) => return self.io_error(e),
        };

        Ok(Some(self.build_token(
            TokenValue::CharLiteral(res),
            format!("'{}'", res).as_str(),
        )))
    }

    fn read_operator(&mut self) -> Result<Option<Token>, Error> {
        match self.next_char() {
            Ok(None) => Ok(None),
            Ok(Some(c1)) => {
                match c1 {
                    '=' | '*' | '/' | '%' | '^' | '!' => {
                        // can be c or c=
                        let c2 = match self.next_char() {
                            Ok(v) => v,
                            Err(e) => return self.io_error(e),
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
                                Ok(v) => v,
                                Err(e) => return self.io_error(e),
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
                            Ok(v) => v,
                            Err(e) => return self.io_error(e),
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
                    _ => self.error(format!(
                        "unexpected character '{}' while reading operator token",
                        c1
                    )),
                }
            }
            Err(e) => self.io_error(e),
        }
    }

    fn build_operator(&mut self, op: &str) -> Result<Option<Token>, Error> {
        match op {
            "+" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::Add), op))),
            "-" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::Subtract), op))),
            "*" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::Multiply), op))),
            "/" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::Divide), op))),
            "%" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::Modulo), op))),
            "|" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::BinaryOr), op))),
            "&" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::BinaryAnd), op))),
            "^" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::Xor), op))),
            "||" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::LogicalOr), op))),
            "&&" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::LogicalAnd), op))),
            "<<" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::ShiftLeft), op))),
            ">>" => Ok(Some(self.build_token(TokenValue::BinaryOperator(BinaryOperator::ShiftRight), op))),
            "==" => Ok(Some(self.build_token(TokenValue::Comparison(ComparisonOperator::Equals), op))),
            ">" => Ok(Some(self.build_token(TokenValue::Comparison(ComparisonOperator::GreaterThan), op))),
            "<" => Ok(Some(self.build_token(TokenValue::Comparison(ComparisonOperator::LessThan), op))),
            ">=" => Ok(Some(self.build_token(TokenValue::Comparison(ComparisonOperator::GreaterThanOrEquals), op))),
            "<=" => Ok(Some(self.build_token(TokenValue::Comparison(ComparisonOperator::LessThanOrEquals), op))),
            "!=" => Ok(Some(self.build_token(TokenValue::Comparison(ComparisonOperator::NotEquals), op))),
            "!" => Ok(Some(self.build_token(TokenValue::UnaryOperator(UnaryOperator::Not), op))),
            "++" => Ok(Some(self.build_token(TokenValue::UnaryOperator(UnaryOperator::Increment), op))),
            "--" => Ok(Some(self.build_token(TokenValue::UnaryOperator(UnaryOperator::Decrement), op))),
            "=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::Assign), op))),
            "+=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Add)), op))),
            "-=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Subtract)), op))),
            "*=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Multiply)), op))),
            "/=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Divide)), op))),
            "%=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Modulo)), op))),
            "|=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::BinaryOr)), op))),
            "&=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::BinaryAnd)), op))),
            "^=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::Xor)), op))),
            "||=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::LogicalOr)), op))),
            "&&=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::LogicalAnd)), op))),
            "<<=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::ShiftLeft)), op))),
            ">>=" => Ok(Some(self.build_token(TokenValue::Assignment(AssignOperator::AssignAfter(BinaryOperator::ShiftRight)), op))),
            _ => self.internal_error(format!("unknown operator string {}", op)),
        }
    }

    fn read_escape_sequence(&mut self) -> Result<char, Error> {
        match self.next_char() {
            Ok(Some(c)) => match c {
                'r' => return Ok('\n'),
                'n' => return Ok('\n'),
                't' => return Ok('\t'),
                '\\' | '\'' | '"' => return Ok(c),
                // TODO: unicode and hex escape codes
                _ => {
                    return self
                        .error(format!("invalid character in escape sequence: {}", c))
                        .map(|_| ' ')
                }
            },
            Ok(None) => {
                return self
                    .error("EOF reached while reading escape sequence".to_string())
                    .map(|_| ' ')
            }
            Err(e) => return self.io_error(e).map(|_| ' '),
        }
    }

    fn push_char(&mut self, c: char) {
        self.lookahead_buf.push_back(c)
    }

    fn internal_error(&self, msg: String) -> Result<Option<Token>, Error> {
        return Err(Error {
            message: format!(
                "{}:{}:{}: {}",
                self.stream_path, self.stream_line, self.stream_column, msg
            ),
            kind: ErrorKind::Internal,
        });
    }

    fn error(&self, msg: String) -> Result<Option<Token>, Error> {
        return Err(Error {
            message: format!(
                "{}:{}:{}: {}",
                self.stream_path, self.stream_line, self.stream_column, msg
            ),
            kind: ErrorKind::InvalidInput,
        });
    }

    fn io_error(&self, err: io::Error) -> Result<Option<Token>, Error> {
        return Err(Error {
            message: format!(
                "{}:{}:{}: I/O error",
                self.stream_path, self.stream_line, self.stream_column,
            ),
            kind: ErrorKind::IOError(err),
        });
    }
}

fn read_char<R: Read>(r: &mut R) -> io::Result<Option<char>> {
    let mut rune_len = 0;
    let mut rune = [0u8; 4];
    while rune_len < 4 {
        match r.read(&mut rune[rune_len..rune_len + 1]) {
            Ok(0) => {
                if rune_len == 0 {
                    return Ok(None);
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "incomplete UTF-8 codepoint",
                    ));
                }
            }
            Ok(_) => {
                rune_len = rune_len + 1;
            }
            Err(e) => return Err(e),
        };

        match std::str::from_utf8(&rune) {
            Ok(s) => return Ok(Some(s.chars().next().unwrap())),
            Err(_) => (),
        }
    }
    return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "failed to parse UTF-8 data: {:x}{:x}{:x}{:x}",
            rune[0], rune[1], rune[2], rune[3]
        ),
    ));
}

fn is_numeric(c: char, base: u32) -> bool {
    c.to_digit(base).is_some()
}

fn is_operator(c: char) -> bool {
    "+-/*%!&|^=<>".contains(c)
}

#[cfg(test)]
mod test;

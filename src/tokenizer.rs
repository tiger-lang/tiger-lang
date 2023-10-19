use std::io::{self, Read};

pub struct Token {
    pub column: usize,
    pub line: usize,
    pub length: usize,
    pub text: String,
    pub value: TokenValue,
}

pub enum PrefixUnaryOperator {
    Not,
}
pub enum SuffixUnaryOperator {
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
}

pub enum AssignOperator {
    Assign,
    AssignAfter(BinaryOperator),
}

pub enum TokenValue {
    Identifier(String),

    ConstUnsignedInteger(u64),
    ConstSignedInteger(i64),
    ConstFloatingPoint(f64),
    ConstString(String),
    ConstBool(bool),

    PrefixUnaryOperator(PrefixUnaryOperator),
    SuffixUnaryOperator(SuffixUnaryOperator),
    BinaryOperator(BinaryOperator),
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
}

impl Token {}

pub enum ErrorKind {
    InvalidInput,
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
    leftover_char: Option<char>,

    token_buf: Vec<char>,
    token_column: usize,
    token_line: usize,

    state: TokenizerState,
    in_escape_sequence: bool,
}

/// Represents the internal state of the tokenizer state machine
enum TokenizerState {
    Standard,
    Identifier,
    String,
    Number,
    FloatingPointNumber,
    Character,
    SingleLineComment,
    MultiLineComment,
    Operator,
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
            leftover_char: None,
            token_buf: vec![],
            token_column: 1,
            token_line: 1,
            state: TokenizerState::Standard,
            in_escape_sequence: false,
        };
    }

    /// Reads the next token from the stream.
    ///
    /// Returns Ok(None) when EOF has been reached without errors.
    pub fn next(&mut self) -> Result<Option<Token>, Error> {
        loop {
            if self.leftover_char.is_some() {
                match self.consume_character(self.leftover_char.unwrap()) {
                    Ok(_) => todo!(),
                    Err(_) => todo!(),
                }
            }

            match read_char(&mut self.stream) {
                Ok(None) => return self.finish_token(), // EOL reached, attempt to finalize the current token
                Ok(Some(c)) => match self.consume_character(c) {
                    Ok((true, token_complete)) => {
                        self.stream_column += 1;
                        if c == '\n' {
                            self.stream_column = 1;
                            self.stream_line += 1;
                        }
                        if token_complete {
                            return self.finish_token();
                        }
                    }
                    Ok((false, true)) => {
                        self.leftover_char = Some(c);
                        return self.finish_token();
                    }
                    Ok((false, false)) => {
                        panic!("tokenizer did not consume character {} and also didn't complete a token - this is an infinite loop", c);
                    }
                    Err(e) => return Err(e),
                },
                Err(err) => {
                    return Err(self.io_error(err));
                }
            }
        }
    }

    /// finish_token finalizes the current token in self.token_buf.
    ///
    /// Returns Ok(None) if self.token_buf is empty
    fn finish_token(&mut self) -> Result<Option<Token>, Error> {
        if self.in_escape_sequence {
            return Err(
                self.error("unexpected end of token while processing escape sequence".to_string())
            );
        }
        // TODO: process current token
        self.state = TokenizerState::Standard;
        todo!()
    }

    /// consume_character takes a character and runs it through the tokenizer.
    ///
    /// Returns `Ok(consumed, token_completed)` on success
    /// consumed indicates whether or not the character was consumed as part of the current token,
    /// token_completed indicates the end of the current token was reached.
    ///
    /// Returns `Err(Error)` when the provided character was invalid in the current context.
    fn consume_character(&mut self, c: char) -> Result<(bool, bool), Error> {
        match self.state {
            TokenizerState::Standard => {
                let mut token_complete = false;
                match c {
                    c if c == '_' || c.is_alphabetic() => {
                        self.state = TokenizerState::Identifier;
                    }
                    c if c.is_numeric() => self.state = TokenizerState::Number,
                    c if c.is_whitespace() => return Ok((true, false)),
                    '+' | '-' | '*' | '/' | '%' | '^' | '&' | '|' => {
                        self.state = TokenizerState::Operator;
                    }
                    '=' | '!' | '{' | '}' | '[' | ']' | '(' | ')' | ',' | '.' => {
                        token_complete = true;
                    }
                    '\'' => self.state = TokenizerState::Character,
                    '"' => self.state = TokenizerState::String,
                    c => {
                        return Err(Error {
                            message: format!(
                                "unexpected character '{}' at line {} column {}",
                                c, self.stream_line, self.stream_column
                            ),
                            kind: ErrorKind::InvalidInput,
                        })
                    }
                };
                self.token_buf.push(c);
                self.token_column = self.stream_column;
                self.token_line = self.stream_line;
                Ok((true, token_complete))
            }
            TokenizerState::Identifier => self.consume_character_ident(c),
            TokenizerState::String => self.consume_character_string(c),
            TokenizerState::Number => todo!(),
            TokenizerState::FloatingPointNumber => todo!(),
            TokenizerState::Character => todo!(),
            TokenizerState::SingleLineComment => todo!(),
            TokenizerState::MultiLineComment => todo!(),
            TokenizerState::Operator => todo!(),
        }
    }

    fn consume_character_ident(&mut self, c: char) -> Result<(bool, bool), Error> {
        match c {
            c if c.is_alphanumeric() || c == '_' => {
                self.token_buf.push(c);
                Ok((true, false))
            }
            '=' | '!' | '{' | '}' | '[' | ']' | '(' | ')' | ',' | '.' => Ok((false, true)),
            c if c.is_whitespace() => Ok((true, true)),
            _ => Err(self.error(format!("unexpected character '{}' in identifier", c))),
        }
    }

    fn consume_character_string(&mut self, c: char) -> Result<(bool, bool), Error> {
        if self.in_escape_sequence {
            match c {
                't' => self.token_buf.push('\t'),
                'n' => self.token_buf.push('\n'),
                _ => {
                    return Err(self.error(format!("unexpected character {} in escape sequence", c)))
                }
            }
            self.in_escape_sequence = false;
            return Ok((true, false));
        }
        match c {
            '"' => return Ok((true, true)),
            '\\' => {
                self.in_escape_sequence = true;
            }
            '\n' => {
                return Err(
                    self.error("unexpected unescaped newline inside string constant".to_string())
                );
            }
            _ => {
                self.token_buf.push(c);
            }
        }
        Ok((true, false))
    }

    fn error(&self, msg: String) -> Error {
        return Error {
            message: format!(
                "{}:{}:{}: {}",
                self.stream_path, self.stream_line, self.stream_column, msg
            ),
            kind: ErrorKind::InvalidInput,
        };
    }

    fn io_error(&self, err: io::Error) -> Error {
        return Error {
            message: format!(
                "{}:{}:{}: I/O error",
                self.stream_path, self.stream_line, self.stream_column,
            ),
            kind: ErrorKind::IOError(err),
        };
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

#[cfg(test)]
mod test;

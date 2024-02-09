use crate::{
    lang::Module,
    tokenizer::{self, Token, TokenStream},
};
use std::io::Read;

pub struct Parser {
    module: Module,
}

pub struct Error {
    pub message: String,
    pub kind: ErrorKind,

    pub line: usize,
    pub column: usize,
    pub source: String,
}

impl Error {
    fn new<R: Read>(ts: &TokenStream<R>, kind: ErrorKind, message: String) -> Self {
        let (path, line, col) = ts.position();
        Self {
            message: message,
            kind: kind,
            line: line,
            column: col,
            source: path,
        }
    }

    fn unexpected_token(t: Token, message: Option<String>) -> Self {
        Self {
            message: message.unwrap_or(String::new()),
            line: t.line,
            column: t.column,
            source: t.path.clone(),
            kind: ErrorKind::UnexpectedToken(t),
        }
    }
}

pub enum ErrorKind {
    Nop,
    TokenizerError(tokenizer::Error),
    UnexpectedToken(Token),
    UnexpectedEOF,
}

impl Parser {
    pub fn new(module_id: String) -> Self {
        Self {
            module: Module::new(module_id),
        }
    }

    pub fn add_source<R: Read>(&mut self, r: R, path: Option<String>) -> Result<(), Error> {
        let mut t = TokenStream::new(r, path);

        self.maybe_parse_use_block(&mut t)?;
        self.maybe_parse_const_block(&mut t)?;
        self.maybe_parse_var_block(&mut t)?;

        while !t.is_empty() {
            self.maybe_parse_func(&mut t)?;
        }

        Ok(())
    }

    pub fn finalize(self) -> Result<Module, Error> {
        todo!()
    }

    fn maybe_parse_use_block<R: Read>(
        &self,
        token_stream: &mut TokenStream<R>,
    ) -> Result<(), Error> {
        if let Some(t) = token_stream.peek() {
            match t {
                Ok(t) => match t.value {
                    tokenizer::TokenValue::KeywordUse => (),
                    tokenizer::TokenValue::KeywordFunc
                    | tokenizer::TokenValue::KeywordTest
                    | tokenizer::TokenValue::KeywordVar
                    | tokenizer::TokenValue::KeywordConst => return Ok(()), // no use block present
                    _ => return Err(Error::unexpected_token(t, None)),
                },
                Err(e) => return Err(e.into()),
            }
        } else {
            return Ok(()); // EOF
        }

        _ = token_stream.next_token(); // consume peeked use keyword

        match token_stream.next_token() {
            Some(Ok(t)) => {
                if t.value != tokenizer::TokenValue::OpenParen {
                    return Err(Error::unexpected_token(t, None));
                }
            }
            Some(Err(e)) => return Err(e.into()),
            None => {
                return Err(Error::new(
                    token_stream,
                    ErrorKind::UnexpectedEOF,
                    "while parsing use block".into(),
                ))
            }
        }

        // Read sequence of x.y.z as w <newline> until ')'

        todo!()
    }

    fn maybe_parse_const_block<R: Read>(
        &self,
        token_stream: &mut TokenStream<R>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn maybe_parse_var_block<R: Read>(
        &self,
        token_stream: &mut TokenStream<R>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn maybe_parse_func<R: Read>(&self, token_stream: &mut TokenStream<R>) -> Result<(), Error> {
        todo!()
    }
}

fn skip_until<R: Read, F>(token_stream: &mut TokenStream<R>, matcher: F) -> Result<(), Error>
where
    F: Fn(&Token) -> bool,
{
    while !token_stream.is_empty() {
        if let Some(t) = token_stream.peek() {
            match t {
                Ok(t) => {
                    if matcher(&t) {
                        return Ok(());
                    }
                    _ = token_stream.next_token()
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    Ok(())
}

impl From<tokenizer::Error> for Error {
    fn from(value: tokenizer::Error) -> Self {
        Error {
            message: "tokenizer error".to_string(),
            line: value.line,
            column: value.column,
            source: value.source.clone(),
            kind: ErrorKind::TokenizerError(value),
        }
    }
}

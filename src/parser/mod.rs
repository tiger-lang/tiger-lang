use crate::{
    lang::{Import, Module},
    tokenizer::{self, Token, TokenStream, TokenValue},
};
use std::io::Read;

mod error;
pub use error::{Error, ErrorKind};

pub struct Parser {
    module: Module,
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
        Ok(self.module)
    }

    fn complete_import<R: Read>(
        &mut self,
        token_stream: &mut TokenStream<R>,
        current_parts: &mut Vec<Token>,
        alias: &str,
    ) -> Result<(), Error> {
        let import = current_parts
            .iter()
            .map(|t| t.text.clone())
            .collect::<Vec<String>>()
            .join("");
        if import.ends_with(".") || import.starts_with(".") {
            return Err(Error::new(
                token_stream,
                ErrorKind::InvalidImport(import),
                "import cannot start or end with '.'".into(),
            ));
        }
        if import.is_empty() {
            return Err(Error::new(
                token_stream,
                ErrorKind::InvalidImport(import),
                "empty import".into(),
            ));
        }
        let first_token = current_parts[0].clone();
        current_parts.clear();

        let import = Import::new(import, alias.into(), first_token);
        let ident = import.local_alias.clone();

        self.module
            .import(import)
            .map_err(|_| Error::redefined_symbol(token_stream, &ident))?;
        Ok(())
    }

    fn maybe_parse_use_block<R: Read>(
        &mut self,
        token_stream: &mut TokenStream<R>,
    ) -> Result<(), Error> {
        if scan_for_keyword(
            token_stream,
            TokenValue::KeywordUse,
            vec![
                TokenValue::KeywordConst,
                TokenValue::KeywordFunc,
                TokenValue::KeywordTest,
                TokenValue::KeywordConst,
            ],
        )? == false
        {
            return Ok(());
        }
        skip_while(token_stream, is_newline)?;

        consume_token(
            token_stream,
            |t| t.value == TokenValue::OpenParen,
            "use keyword should be followed by a `(`".into(),
        )?;

        // Read sequence of x.y.z as w <newline> until ')'
        let mut current_parts = vec![];

        while let Some(t) = token_stream.next_token() {
            let t = t?;
            match t.value {
                TokenValue::CloseParen => {
                    if !current_parts.is_empty() {
                        self.complete_import(token_stream, &mut current_parts, "")?;
                    }
                    return Ok(());
                }
                TokenValue::KeywordAs => {
                    let alias = consume_token(
                        token_stream,
                        is_identifier,
                        "while parsing import alias".into(),
                    )?;
                    self.complete_import(token_stream, &mut current_parts, &alias.text)?;
                    ensure_next_token(
                        token_stream,
                        |t| match t.value {
                            TokenValue::Comma | TokenValue::Newline => true,
                            _ => false,
                        },
                        "while parsing import alias".into(),
                    )?;
                }
                TokenValue::Newline => {
                    if !current_parts.is_empty() {
                        self.complete_import(token_stream, &mut current_parts, "")?;
                    }
                }
                TokenValue::Comma => self.complete_import(token_stream, &mut current_parts, "")?,
                TokenValue::Identifier(_) | TokenValue::Dot => current_parts.push(t),
                _ => {
                    return Err(Error::unexpected_token(
                        t,
                        "while parsing `use` block".into(),
                    ))
                }
            }
        }

        return Err(Error::new(
            token_stream,
            ErrorKind::UnexpectedEOF,
            "missing ')' after 'use' block".into(),
        ));
    }

    fn maybe_parse_const_block<R: Read>(
        &mut self,
        token_stream: &mut TokenStream<R>,
    ) -> Result<(), Error> {
        if scan_for_keyword(
            token_stream,
            TokenValue::KeywordConst,
            vec![
                TokenValue::KeywordFunc,
                TokenValue::KeywordTest,
                TokenValue::KeywordConst,
            ],
        )? == false
        {
            return Ok(()); // No const block
        }

        skip_while(token_stream, is_newline)?;

        consume_token(
            token_stream,
            |t| t.value == TokenValue::OpenParen,
            "use keyword should be followed by a `(`".into(),
        )?;
        todo!()
    }

    fn maybe_parse_var_block<R: Read>(
        &mut self,
        token_stream: &mut TokenStream<R>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn maybe_parse_func<R: Read>(
        &mut self,
        token_stream: &mut TokenStream<R>,
    ) -> Result<(), Error> {
        todo!()
    }
}

fn is_identifier(t: &Token) -> bool {
    if let TokenValue::Identifier(_) = t.value {
        true
    } else {
        false
    }
}

fn is_newline(t: &Token) -> bool {
    match t.value {
        TokenValue::Newline => true,
        _ => false,
    }
}

fn ensure_next_token<R: Read, F>(
    token_stream: &mut TokenStream<R>,
    matcher: F,
    error_message: String,
) -> Result<(), Error>
where
    F: Fn(&Token) -> bool,
{
    if let Some(t) = token_stream.peek() {
        let t = t?;
        if matcher(&t) {
            Ok(())
        } else {
            Err(Error::unexpected_token(t, error_message))
        }
    } else {
        Err(Error::new(
            token_stream,
            ErrorKind::UnexpectedEOF,
            error_message,
        ))
    }
}

fn consume_token<R: Read, F>(
    token_stream: &mut TokenStream<R>,
    matcher: F,
    error_message: String,
) -> Result<Token, Error>
where
    F: Fn(&Token) -> bool,
{
    if let Some(t) = token_stream.next_token() {
        let t = t?;

        if matcher(&t) {
            return Ok(t);
        } else {
            return Err(Error::unexpected_token(t, error_message));
        }
    } else {
        return Err(Error::new(
            token_stream,
            ErrorKind::UnexpectedEOF,
            "".into(),
        ));
    }
}

fn scan_for_keyword<R: Read>(
    token_stream: &mut TokenStream<R>,
    keyword: TokenValue,
    non_error_keywords: Vec<TokenValue>,
) -> Result<bool, Error> {
    skip_while(token_stream, is_newline)?;

    if let Some(t) = token_stream.peek() {
        let t = t?;
        if t.value == keyword {
            _ = token_stream.next_token();
            return Ok(true);
        }
        if non_error_keywords.contains(&t.value) {}
        return Err(Error::unexpected_token(
            t,
            format!("while looking for `{}` block", keyword),
        ));
    } else {
        return Ok(false); // EOF
    }
}

fn skip_while<R: Read, F>(token_stream: &mut TokenStream<R>, matcher: F) -> Result<(), Error>
where
    F: Fn(&Token) -> bool,
{
    skip_until(token_stream, |t| !matcher(t))
}

fn skip_until<R: Read, F>(token_stream: &mut TokenStream<R>, matcher: F) -> Result<(), Error>
where
    F: Fn(&Token) -> bool,
{
    while !token_stream.is_empty() {
        if let Some(t) = token_stream.peek() {
            let t = t?;
            if matcher(&t) {
                return Ok(());
            }
            _ = token_stream.next_token()
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

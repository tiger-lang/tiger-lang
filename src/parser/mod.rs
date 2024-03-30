use crate::{
    lang::{Expression, Module, Symbol, Type},
    tokenizer::{self, Token, TokenStream, TokenValue},
};
use std::io::Read;

mod error;
pub use error::{Error, ErrorKind};

pub type Result<T> = std::result::Result<T, Error>;

mod expression;

pub(self) mod token_matcher;

pub struct Parser {
    module: Module,
}

impl Parser {
    pub fn new(module_id: String) -> Self {
        Self {
            module: Module::new(module_id),
        }
    }

    pub fn add_source<R: Read>(&mut self, r: R, path: Option<String>) -> Result<()> {
        let mut t = TokenStream::new(r, path);

        self.maybe_parse_use_block(&mut t)?;
        self.maybe_parse_const_block(&mut t)?;
        self.maybe_parse_var_block(&mut t)?;

        while !t.is_empty() {
            self.parse_module_body(&mut t)?;
        }

        Ok(())
    }

    pub fn finalize(self) -> Result<Module> {
        Ok(self.module)
    }

    fn complete_import<R: Read>(
        &mut self,
        token_stream: &mut TokenStream<R>,
        current_parts: &mut Vec<Token>,
        alias: &str,
    ) -> Result<()> {
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

        let ident: String = if alias.is_empty() {
            import.split(".").last().unwrap_or("").into()
        } else {
            alias.into()
        };

        let import = Symbol::new_import(import, first_token);

        self.module
            .define(ident.clone(), import)
            .map_err(|_| Error::redefined_symbol(token_stream, &ident))?;
        Ok(())
    }

    fn maybe_parse_use_block<R: Read>(&mut self, token_stream: &mut TokenStream<R>) -> Result<()> {
        if scan_for_keyword(
            token_stream,
            TokenValue::KeywordUse,
            vec![
                TokenValue::KeywordFunc,
                TokenValue::KeywordTest,
                TokenValue::KeywordStruct,
                TokenValue::KeywordConst,
                TokenValue::KeywordVar,
            ],
        )? == false
        {
            return Ok(());
        }
        skip_while(token_stream, token_matcher::newline)?;

        consume_token(
            token_stream,
            |t| t.value == TokenValue::OpenBrace,
            "use keyword should be followed by a `{`".into(),
        )?;

        // Read sequence of x.y.z as w <newline> until ')'
        let mut current_parts = vec![];

        while let Some(t) = token_stream.next_token() {
            let t = t?;
            match t.value {
                TokenValue::CloseBrace => {
                    if !current_parts.is_empty() {
                        self.complete_import(token_stream, &mut current_parts, "")?;
                    }
                    return Ok(());
                }
                TokenValue::KeywordAs => {
                    let alias = consume_token(
                        token_stream,
                        token_matcher::identifier,
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
    ) -> Result<()> {
        if scan_for_keyword(
            token_stream,
            TokenValue::KeywordConst,
            vec![
                TokenValue::KeywordFunc,
                TokenValue::KeywordTest,
                TokenValue::KeywordStruct,
                TokenValue::KeywordVar,
            ],
        )? == false
        {
            return Ok(()); // No const block
        }

        let consts = parse_declaration_block(token_stream)?;
        for decl in consts {
            let c = Symbol::new_const(Type::from(decl.ttype), decl.value, decl.first_token);
            self.module
                .define(decl.identifier.clone(), c)
                .map_err(|_| Error::redefined_symbol(token_stream, &decl.identifier))?;
        }

        Ok(())
    }

    fn maybe_parse_var_block<R: Read>(&mut self, token_stream: &mut TokenStream<R>) -> Result<()> {
        if scan_for_keyword(
            token_stream,
            TokenValue::KeywordConst,
            vec![
                TokenValue::KeywordFunc,
                TokenValue::KeywordTest,
                TokenValue::KeywordStruct,
            ],
        )? == false
        {
            return Ok(()); // No const block
        }

        let vars = parse_declaration_block(token_stream)?;
        for decl in vars {
            let v = Symbol::new_var(Type::from(decl.ttype), decl.value, decl.first_token);
            self.module
                .define(decl.identifier.clone(), v)
                .map_err(|_| Error::redefined_symbol(token_stream, &decl.identifier))?;
        }

        Ok(())
    }

    fn parse_module_body<R: Read>(&mut self, token_stream: &mut TokenStream<R>) -> Result<()> {
        skip_while(token_stream, token_matcher::newline)?;

        if let Some(t) = token_stream.peek() {
            let t = t?;
        }
        todo!()
    }
}

fn ensure_next_token<R: Read, F>(
    token_stream: &mut TokenStream<R>,
    matcher: F,
    error_message: String,
) -> Result<()>
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
) -> Result<Token>
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
) -> Result<bool> {
    skip_while(token_stream, token_matcher::newline)?;

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

fn skip_while<R: Read, F>(token_stream: &mut TokenStream<R>, matcher: F) -> Result<()>
where
    F: Fn(&Token) -> bool,
{
    skip_until(token_stream, |t| !matcher(t))
}

fn skip_until<R: Read, F>(token_stream: &mut TokenStream<R>, matcher: F) -> Result<()>
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

struct Declaration {
    identifier: String,
    ttype: Type,
    value: Expression,
    first_token: Token,
}

fn parse_type<R: Read>(token_stream: &mut TokenStream<R>) -> Result<Type> {
    // TODO: list types
    let ttype = consume_token(
        token_stream,
        token_matcher::identifier,
        "expected type definition".into(),
    )?;
    match ttype.value {
        TokenValue::Identifier(s) => Ok(s.into()),
        _ => unreachable!(),
    }
}

/// parse_declaration_block parses the "body" of a var or const block, including the opening and closing brace.
fn parse_declaration_block<R: Read>(token_stream: &mut TokenStream<R>) -> Result<Vec<Declaration>> {
    skip_while(token_stream, token_matcher::newline)?;

    consume_token(
        token_stream,
        |t| t.value == TokenValue::OpenBrace,
        "const and var blocks should start with a `{`".into(),
    )?;

    let mut res = vec![];
    while let Some(t) = token_stream.peek() {
        let t = t?;
        if t.value == TokenValue::CloseBrace {
            break;
        }
        let ident = consume_token(
            token_stream,
            token_matcher::identifier,
            "expected identifier or `}`".into(),
        )?;

        let ttype = parse_type(token_stream)?;

        consume_token(
            token_stream,
            |t| match t.value {
                TokenValue::Assignment(tokenizer::AssignOperator::Assign) => true,
                _ => false,
            },
            "expected `=`".into(),
        )?;
        let value = expression::parse(token_stream, &token_matcher::newline)?;
        res.push(Declaration {
            identifier: match ident.value {
                TokenValue::Identifier(ref s) => s.into(),
                _ => unreachable!(),
            },
            ttype,
            value,
            first_token: ident,
        });
    }
    consume_token(
        token_stream,
        |t| t.value == TokenValue::CloseBrace,
        "expected `}` at the end of a const block".into(),
    )?;
    Ok(res)
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

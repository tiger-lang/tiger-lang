use std::io::Read;

use crate::tokenizer::{self, TokenStream, Token};

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub kind: ErrorKind,

    pub line: usize,
    pub column: usize,
    pub source: String,
}

impl Error {
    pub(super) fn new<R: Read>(ts: &TokenStream<R>, kind: ErrorKind, message: String) -> Self {
        let (path, line, col) = ts.position();
        Self {
            message,
            kind,
            line,
            column: col,
            source: path,
        }
    }

    pub(super) fn redefined_symbol<R: Read>(ts: &TokenStream<R>, ident: &str) -> Self {
	let (path, line, col) = ts.position();
        Self {
            message: format!("`{}` already defined", ident),
            kind: ErrorKind::SymbolRedefined(ident.into()),
            line,
            column: col,
            source: path,
        }
    }

    pub(super) fn unexpected_token(t: Token, message: String) -> Self {
        Self {
            message: message,
            line: t.line,
            column: t.column,
            source: t.path.clone(),
            kind: ErrorKind::UnexpectedToken(t),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let ErrorKind::TokenizerError(e) = &self.kind {
	    return e.fmt(f);
	}
        f.write_fmt(format_args!(
            "{}:{}:{}: {}",
            self.source, self.line, self.column, self.kind
        ))?;
	if self.message.is_empty() {
	    return Ok(());
	}
	f.write_fmt(format_args!(" ({})", self.message))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Nop,
    TokenizerError(tokenizer::Error),
    UnexpectedToken(Token),
    UnexpectedEOF,
    InvalidImport(String),
    SymbolRedefined(String),
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Nop => f.write_str("no error"),
            ErrorKind::TokenizerError(e) => e.fmt(f),
            ErrorKind::UnexpectedToken(t) => f.write_fmt(format_args!("unexpected token `{}`", t.text)),
            ErrorKind::UnexpectedEOF => f.write_str("unexpected EOF"),
            ErrorKind::InvalidImport(imp) => f.write_fmt(format_args!("invalid import `{}`", imp)),
            ErrorKind::SymbolRedefined(s) => f.write_fmt(format_args!("`{}` is already defined", s)),
        }
    }
}

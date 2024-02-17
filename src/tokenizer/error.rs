use std::io;

#[derive(Debug)]
pub enum ErrorKind {
    InvalidInput,
    Internal,
    IOError(io::Error),
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub kind: ErrorKind,

    pub line: usize,
    pub column: usize,
    pub source: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidInput => f.write_str("invalid input"),
            ErrorKind::Internal => f.write_str("internal tokenizer error"),
            ErrorKind::IOError(e) => f.write_fmt(format_args!("I/O error: {}", e)),
        }
    }
}

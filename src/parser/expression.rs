use std::io::Read;

use crate::{
    lang::{Expression, ExpressionValue},
    tokenizer::{BinaryOperator, Token, TokenStream, TokenValue, UnaryOperator},
};

use super::{token_matcher, Error, ErrorKind};

pub fn parse<R: Read, F>(ts: &mut TokenStream<R>, terminator: &F) -> Result<Expression, Error>
where
    F: Fn(&Token) -> bool,
{
    let first_token = match ts.peek() {
        Some(t) => t?,
        None => {
            return Err(Error::new(
                ts,
                ErrorKind::UnexpectedEOF,
                "unexpected EOF, expression expected".into(),
            ));
        }
    };

    let mut first_subexp = match &first_token.value {
        TokenValue::Identifier(_) => parse_identifier(ts, terminator),
        TokenValue::IntegerLiteral(v) => Ok(Expression::literal_int(
            v.to_owned(),
            ts.next_token().unwrap()?,
        )),
        TokenValue::FloatingPointLiteral(v) => Ok(Expression::literal_float(
            v.to_owned(),
            ts.next_token().unwrap()?,
        )),
        TokenValue::StringLiteral(v) => Ok(Expression::literal_string(
            v.to_owned(),
            ts.next_token().unwrap()?,
        )),
        TokenValue::CharLiteral(v) => Ok(Expression::literal_char(
            v.to_owned(),
            ts.next_token().unwrap()?,
        )),
        TokenValue::BoolLiteral(v) => Ok(Expression::literal_bool(
            v.to_owned(),
            ts.next_token().unwrap()?,
        )),
        TokenValue::UnaryOperator(v) => {
            _ = ts.next_token(); // Pop operator
            let sub_expression = parse(ts, terminator)?;
            match v {
                // Note: the Minus and Plus case currently can't be reached, because the
                // tokenizer converts all '+' and '-' signs into BinaryOperator tokens.
                UnaryOperator::Not => {
                    Ok(Expression::unary_not(sub_expression, first_token.clone()))
                }
                UnaryOperator::Minus => {
                    Ok(Expression::unary_minus(sub_expression, first_token.clone()))
                }
                UnaryOperator::Plus => {
                    Ok(Expression::unary_plus(sub_expression, first_token.clone()))
                }
            }
        }
        TokenValue::BinaryOperator(BinaryOperator::Subtract) => {
            _ = ts.next_token(); // Pop operator
            let sub_expression = parse(ts, terminator)?;
            Ok(Expression::unary_minus(sub_expression, first_token.clone()))
        }
        TokenValue::BinaryOperator(BinaryOperator::Add) => {
            _ = ts.next_token(); // Pop operator
            let sub_expression = parse(ts, terminator)?;
            Ok(Expression::unary_plus(sub_expression, first_token.clone()))
        }
        TokenValue::OpenParen => {
            _ = ts.next_token(); // Pop '('
            let sub_expression = parse(ts, &token_matcher::close_paren)?;
            super::consume_token(ts, token_matcher::close_paren, "".into())?;
            Ok(sub_expression)
        }
        _ => {
            return Err(Error::unexpected_token(
                first_token,
                "expecting an expression".into(),
            ))
        }
    }?;

    while let Some(t) = ts.peek() {
        let t = t?;
        if terminator(&t) {
            return Ok(first_subexp);
        }
        match (&first_subexp.value, &t.value) {
            (ExpressionValue::Identifier(_), TokenValue::OpenParen) => {
                // Parsing a function call
                _ = ts.next_token();
                let mut args = Vec::new();
                loop {
                    let arg = parse(
                        ts,
                        &token_matcher::either(token_matcher::close_paren, token_matcher::comma),
                    )?;
                    args.push(arg);
                    if let Some(t) = ts.peek() {
                        let t = t?;
                        match &t.value {
                            TokenValue::CloseParen => {
                                _ = ts.next_token();
                                break;
                            }
                            TokenValue::Comma => {
                                _ = ts.next_token();
                            }
                            _ => {
                                return Err(Error::unexpected_token(
                                    t,
                                    "while parsing function call argument list".into(),
                                ))
                            }
                        }
                    } else {
                        // Should not happen, parse above would have returned unexpected EOF error
                        return Err(Error::new(
                            ts,
                            ErrorKind::UnexpectedEOF,
                            "unexpected EOF, expression expected".into(),
                        ));
                    }
                }
                first_subexp = Expression::function_call(first_subexp, args);
            } // Function call
            (_, TokenValue::Dot) => {
                // Struct member access
                // TODO
                return Err(Error::unexpected_token(
                    t,
                    "member access is only supported for identifier expressions".into(),
                ));
            }
            (_, TokenValue::BinaryOperator(op)) => todo!(),
            (_, TokenValue::OpenBracket) => {
                // List member access
                return Err(Error::unexpected_token(
                    t,
                    "list member access is not implemented yet".into(),
                ));
            }
            (_, _) => {
                return Err(Error::unexpected_token(
                    t,
                    "while parsing expression".into(),
                ))
            }
        }
    }
    return Ok(first_subexp);
}

fn parse_identifier<R: Read, F>(ts: &mut TokenStream<R>, terminator: F) -> Result<Expression, Error>
where
    F: Fn(&Token) -> bool,
{
    let first_token: Token = match ts.peek() {
        Some(t) => t?,
        None => {
            return Err(Error::new(
                ts,
                ErrorKind::UnexpectedEOF,
                "unexpected EOF, identifier expected".into(),
            ))
        }
    };

    let mut words = Vec::<String>::new();
    let mut must_dot = false;
    while let Some(t) = ts.peek() {
        let t = t?;
        match &t.value {
            TokenValue::Identifier(i) => {
                if !must_dot {
                    _ = ts.next_token();
                    words.push(i.clone());
                    must_dot = true;
                } else {
                    Error::unexpected_token(t, "expected `.` or end of identifier".into());
                }
            }

            TokenValue::Dot => {
                if must_dot {
                    _ = ts.next_token();
                    must_dot = false;
                } else {
                    Error::unexpected_token(t, "while parsing identifier".into());
                }
            }

            TokenValue::UnaryOperator(_)
            | TokenValue::BinaryOperator(_)
            | TokenValue::Assignment(_)
            | TokenValue::Comma
            | TokenValue::Newline
            | TokenValue::OpenBracket => break,

            _ => {
                if terminator(&t) {
                    break;
                } else {
                    return Err(Error::unexpected_token(
                        t,
                        "while parsing identifier".into(),
                    ));
                }
            }
        }
    }

    if words.is_empty() {
        if let Some(t) = ts.peek() {
            let t = t?;
            return Err(Error::unexpected_token(t, "expected identifier".into()));
        } else {
            return Err(Error::new(
                ts,
                ErrorKind::UnexpectedEOF,
                "unexpected EOF, identifier expected".into(),
            ));
        }
    }

    Ok(Expression::identifier(words, first_token))
}

fn test() {
    let function_call = parse!(func: identifier);
}

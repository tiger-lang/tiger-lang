use crate::tokenizer::{BinaryOperator, Token, TokenValue};

pub fn identifier(t: &Token) -> bool {
    if let TokenValue::Identifier(_) = t.value {
        true
    } else {
        false
    }
}

pub fn newline(t: &Token) -> bool {
    match t.value {
        TokenValue::Newline => true,
        _ => false,
    }
}

pub fn comma(t: &Token) -> bool {
    match t.value {
        TokenValue::Comma => true,
        _ => false,
    }
}

pub fn close_paren(t: &Token) -> bool {
    match t.value {
        TokenValue::CloseParen => true,
        _ => false,
    }
}

pub fn either<F1, F2>(term1: F1, term2: F2) -> Box<dyn Fn(&Token) -> bool>
where
    F1: Fn(&Token) -> bool + 'static,
    F2: Fn(&Token) -> bool + 'static,
{
    Box::new(move |t| term1(t) || term2(t))
}

pub fn term_delimiter_for<F>(t: &Token) -> Box<dyn Fn(&Token) -> bool> {
    let p1 = OperatorPrecedence::from(t);

    Box::new(move |t| {
        let p2 = OperatorPrecedence::from(t);
        p2 > p1
    })
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
enum OperatorPrecedence {
    NotAnOperator,
    UnaryOperator,
    Multiplication,
    Sum,
    BitwiseShift,
    Comparison,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
}

impl From<&Token> for OperatorPrecedence {
    fn from(t: &Token) -> Self {
        match &t.value {
            TokenValue::UnaryOperator(_) => Self::UnaryOperator,
            TokenValue::BinaryOperator(b) => match b {
                BinaryOperator::Add | BinaryOperator::Subtract => Self::Sum,
                BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => {
                    Self::Multiplication
                }
                BinaryOperator::BinaryOr => Self::BitwiseOr,
                BinaryOperator::BinaryAnd => Self::BitwiseAnd,
                BinaryOperator::Xor => Self::BitwiseXor,
                BinaryOperator::LogicalOr => Self::LogicalOr,
                BinaryOperator::LogicalAnd => Self::LogicalAnd,
                BinaryOperator::ShiftLeft | BinaryOperator::ShiftRight => Self::BitwiseShift,
                BinaryOperator::Equals
                | BinaryOperator::GreaterThan
                | BinaryOperator::LessThan
                | BinaryOperator::GreaterThanOrEquals
                | BinaryOperator::LessThanOrEquals
                | BinaryOperator::NotEquals => Self::Comparison,
            },
            _ => Self::NotAnOperator,
        }
    }
}

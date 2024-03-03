use crate::tokenizer::{BinaryOperator, Token, UnaryOperator};

#[derive(Clone)]
pub struct Expression {
    pub value: ExpressionValue,
    first_token: Token,
}

#[derive(Clone)]
pub struct Identifier {
    namespace: Vec<String>,
    name: String,
}

#[derive(Clone)]
pub struct FunctionCall {
    function: Box<Expression>,
    args: Vec<Expression>,
}

#[derive(Clone)]
pub struct BinOp {
    operator: BinaryOperator,
    operands: Vec<Expression>,
}

#[derive(Clone)]
pub struct UnOp {
    operator: UnaryOperator,
    operand: Box<Expression>,
}

#[derive(Clone)]
pub enum Literal {
    Integer(i128),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
}

#[derive(Clone)]
pub enum ExpressionValue {
    Identifier(Identifier),
    FunctionCall(FunctionCall),
    BinaryOperation(BinOp),
    UnaryOperation(UnOp),
    Literal(Literal),
}

impl Expression {
    pub fn identifier(words: Vec<String>, first_token: Token) -> Self {
        let mut words = words;
        let ident = words.pop().unwrap_or("".into());
        Self {
            value: ExpressionValue::Identifier(Identifier {
                namespace: words,
                name: ident,
            }),
            first_token,
        }
    }

    pub fn literal_string(v: String, first_token: Token) -> Self {
        Self {
            value: ExpressionValue::Literal(Literal::String(v)),
            first_token,
        }
    }

    pub fn literal_char(v: char, first_token: Token) -> Self {
        Self {
            value: ExpressionValue::Literal(Literal::Char(v)),
            first_token,
        }
    }

    pub fn literal_int(v: i128, first_token: Token) -> Self {
        Self {
            value: ExpressionValue::Literal(Literal::Integer(v)),
            first_token,
        }
    }

    pub fn literal_float(v: f64, first_token: Token) -> Self {
        Self {
            value: ExpressionValue::Literal(Literal::Float(v)),
            first_token,
        }
    }

    pub fn literal_bool(v: bool, first_token: Token) -> Self {
        Self {
            value: ExpressionValue::Literal(Literal::Bool(v)),
            first_token,
        }
    }

    pub fn unary_plus(operand: Expression, first_token: Token) -> Self {
        Self {
            value: ExpressionValue::UnaryOperation(UnOp {
                operator: UnaryOperator::Plus,
                operand: Box::new(operand),
            }),
            first_token,
        }
    }

    pub fn unary_minus(operand: Expression, first_token: Token) -> Self {
        Self {
            value: ExpressionValue::UnaryOperation(UnOp {
                operator: UnaryOperator::Minus,
                operand: Box::new(operand),
            }),
            first_token,
        }
    }

    pub fn unary_not(operand: Expression, first_token: Token) -> Self {
        Self {
            value: ExpressionValue::UnaryOperation(UnOp {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
            }),
            first_token,
        }
    }

    pub fn function_call(function: Expression, args: Vec<Expression>) -> Self {
        let first_token = function.first_token.clone();
        Self {
            value: ExpressionValue::FunctionCall(FunctionCall {
                function: Box::new(function),
                args,
            }),
            first_token,
        }
    }
}

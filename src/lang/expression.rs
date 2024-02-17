use crate::tokenizer::{BinaryOperator, Token, UnaryOperator};

#[derive(Clone)]
pub struct Expression {
    value: ExpressionValue,
    first_token: Token,
}

#[derive(Clone)]
pub struct Identifier {
    namespace: Vec<String>,
    name: String,
}

#[derive(Clone)]
pub struct FunctionCall {
    functionIdent: Identifier,
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
pub enum ExpressionValue {
    Identifier(Identifier),
    FunctionCall(FunctionCall),
    BinaryOperation(BinOp),
    UnaryOperation(UnOp),
}

use std::collections::HashMap;

mod assembly;
mod module;
mod symbol;
mod ttype;

pub use assembly::Assembly;
pub use module::Module;

use crate::tokenizer::Token;

pub enum Symbol {
    Function(Func),
    Variable(Variable),
    Constant(Const),
    Type(StructType),
    Import(Import),
}

pub enum SymbolRef<'a> {
    Function(&'a Func),
    Variable(&'a Variable),
    Constant(&'a Const),
    Import(&'a Import),
    Type(&'a StructType),
}

pub struct Import {
    pub path: String,
    pub signature: Option<FuncSignature>,
    first_token: Token,
}

pub enum Type {
    Text,
    Character,
    Bool,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Float32,
    Float64,
    Struct(String), // Value is the name of the struct type
}

pub struct StructType {
    ident: String,
    first_token: Token,
    fields: Vec<(String, Type)>,
}

pub struct Func {
    signature: FuncSignature,
    constants: HashMap<String, Const>,
    variables: HashMap<String, Type>,
    statements: Vec<Statement>,
    first_token: Token,
}

pub struct FuncSignature {
    args: Vec<Type>,
    return_value: Type,
    first_token: Token,
}

pub struct Statement {
    value: StatementValue,
    first_token: Token,
}

pub enum StatementValue {}

mod expression;
pub use expression::{Expression, ExpressionValue, Identifier};

pub struct Const {
    ttype: Type,
    value: Expression,
    first_token: Token,
}

pub struct Variable {
    ttype: Type,
    initial_value: Expression,
    first_token: Token,
}

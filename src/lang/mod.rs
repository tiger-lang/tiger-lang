use std::collections::HashMap;

mod assembly;
mod module;

pub use assembly::Assembly;
pub use module::Module;

use crate::tokenizer::Token;

pub struct Value {}

pub enum Symbol {
    Function(Func),
    Variable(Type),
    Constant(Const),
}

pub enum SymbolRef<'a> {
    Function(&'a Func),
    Variable(&'a Type),
    Constant(&'a Const),
    Import(&'a Import),
}

pub struct Import {
    ident: String,
    local_alias: String,
    signature: Option<FuncSignature>,
}

pub struct Type {}

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

pub struct Statement {}

pub struct Const {
    ttype: Type,
    value: Value,
    first_token: Token,
}

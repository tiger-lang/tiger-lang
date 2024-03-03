use crate::tokenizer::Token;

use super::{Const, Expression, Import, Symbol, Type, Variable};

impl Symbol {
    pub fn new_import(import: String, first_token: Token) -> Self {
        Self::Import(Import {
            path: import,
            signature: None,
            first_token,
        })
    }

    pub fn new_const(ttype: Type, value: Expression, first_token: Token) -> Self {
        Self::Constant(Const {
            ttype,
            value,
            first_token,
        })
    }

    pub fn new_var(ttype: Type, value: Expression, first_token: Token) -> Self {
        Self::Variable(Variable {
            ttype,
            initial_value: value,
            first_token,
        })
    }
}

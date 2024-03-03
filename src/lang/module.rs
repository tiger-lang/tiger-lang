use std::collections::HashMap;

use super::{Const, Func, FuncSignature, Import, StructType, Symbol, SymbolRef, Variable};

pub struct Module {
    identifier: String,

    functions: HashMap<String, Func>,
    constants: HashMap<String, Const>,
    variables: HashMap<String, Variable>,
    types: HashMap<String, StructType>,

    imports: HashMap<String, Import>,
    exports: HashMap<String, FuncSignature>,
}

impl Module {
    pub fn new(ident: String) -> Self {
        Self {
            identifier: ident,
            constants: HashMap::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
            imports: HashMap::new(),
            exports: HashMap::new(),
        }
    }

    pub fn lookup(&self, ident: &str) -> Option<SymbolRef> {
        if let Some(func) = self.functions.get(ident) {
            return Some(SymbolRef::Function(func));
        }

        if let Some(cst) = self.constants.get(ident) {
            return Some(SymbolRef::Constant(cst));
        }

        if let Some(var) = self.variables.get(ident) {
            return Some(SymbolRef::Variable(var));
        }

        if let Some(ttype) = self.types.get(ident) {
            return Some(SymbolRef::Type(ttype));
        }

        if let Some(import) = self.imports.get(ident) {
            return Some(SymbolRef::Import(import));
        }

        None
    }

    pub fn undefine(&mut self, ident: &str) -> Option<Symbol> {
        if let Some(func) = self.functions.remove(ident) {
            return Some(Symbol::Function(func));
        }

        if let Some(cst) = self.constants.remove(ident) {
            return Some(Symbol::Constant(cst));
        }

        if let Some(var) = self.variables.remove(ident) {
            return Some(Symbol::Variable(var));
        }

        if let Some(ttype) = self.types.remove(ident) {
            return Some(Symbol::Type(ttype));
        }

        if let Some(import) = self.imports.remove(ident) {
            return Some(Symbol::Import(import));
        }

        None
    }

    pub fn define(&mut self, ident: String, symb: Symbol) -> Result<(), ()> {
        if let Some(_) = self.lookup(&ident) {
            return Err(()); // Already defined
        }

        match symb {
            Symbol::Function(f) => {
                self.functions.insert(ident, f);
            }
            Symbol::Variable(v) => {
                self.variables.insert(ident, v);
            }
            Symbol::Constant(c) => {
                self.constants.insert(ident, c);
            }
            Symbol::Type(t) => {
                self.types.insert(ident, t);
            }
            Symbol::Import(i) => {
                self.imports.insert(ident, i);
            }
        }

        Ok(())
    }

    pub fn redefine(&mut self, ident: String, symb: Symbol) -> Option<Symbol> {
        let res = self.undefine(&ident).map(|s| s);
        self.define(ident, symb);
        res
    }
}

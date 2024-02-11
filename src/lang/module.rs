use std::collections::HashMap;

use super::{Const, Func, FuncSignature, Import, Symbol, SymbolRef, Type};

pub struct Module {
    identifier: String,

    functions: HashMap<String, Func>,
    constants: HashMap<String, Const>,
    variables: HashMap<String, Type>,

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

        if let Some(import) = self.imports.get(ident) {
            return Some(SymbolRef::Import(import));
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
        }

        Ok(())
    }

    pub fn redefine(&mut self, ident: String, symb: Symbol) -> Option<Symbol> {
        match symb {
            Symbol::Function(f) => {
                let res = self.functions.insert(ident, f);
                res.map(|f| Symbol::Function(f))
            }
            Symbol::Variable(v) => {
                let res = self.variables.insert(ident, v);
                res.map(|v| Symbol::Variable(v))
            }
            Symbol::Constant(c) => {
                let res = self.constants.insert(ident, c);
                res.map(|c| Symbol::Constant(c))
            }
        }
    }

    pub fn import(&mut self, ident: String, local_alias: String) -> Result<(), ()> {
        let local_alias = if local_alias.is_empty() {
            ident.split(".").last().unwrap_or("").into()
        } else {
            local_alias
        };

        if let Some(_) = self.lookup(&local_alias) {
            return Err(()); // Already defined
        }

        let import = Import {
            ident,
            local_alias: local_alias.clone(),
            signature: None,
        };
        self.imports.insert(local_alias, import);

        Ok(())
    }
}

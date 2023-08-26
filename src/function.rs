use crate::variable::VariableType;

pub struct Function {
    pub name: String,
    pub params: Vec<VariableType>,
}

#[derive(Default)]
pub struct Functions {
    functions: Vec<Function>,
}

impl Functions {
    pub fn lookup(&self, name: &str) -> Option<&Function> {
        for i in &self.functions {
            if i.name == name {
                return Some(i);
            }
        }
        None
    }
    pub fn add(&mut self, name: &str, params: &[VariableType]) {
        self.functions.push(Function {
            name: name.to_string(),
            params: params.to_vec(),
        })
    }
}

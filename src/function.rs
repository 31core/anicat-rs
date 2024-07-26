use crate::variable::VariableType;

#[derive(Default, Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<VariableType>,
}

#[derive(Default, Debug)]
pub struct Functions {
    functions: Vec<Function>,
}

impl Functions {
    pub fn lookup(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|&i| i.name == name)
    }
    pub fn add(&mut self, name: &str, params: &[VariableType]) {
        self.functions.push(Function {
            name: name.to_string(),
            params: params.to_vec(),
        })
    }
}

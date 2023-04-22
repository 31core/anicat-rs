#[derive(Debug)]
pub enum VariableType {
    Uint8,
    Int8,
    Uint16,
    Int16,
    Uint32,
    Int32,
    Uint64,
    Int64,
    Bool,
    Unkown,
}

impl VariableType {
    pub fn from_string(var_type: &str) -> Self {
        match var_type {
            "u8" => Self::Uint8,
            "i8" => Self::Int8,
            "u16" => Self::Uint16,
            "i16" => Self::Int16,
            "u32" => Self::Uint32,
            "i32" => Self::Int32,
            "u64" => Self::Uint64,
            "i64" => Self::Int64,
            "bool" => Self::Bool,
            _ => Self::Unkown,
        }
    }
    pub fn get_size(&self) -> usize {
        match self {
            Self::Uint8 => 1,
            Self::Int8 => 1,
            Self::Uint16 => 2,
            Self::Int16 => 2,
            Self::Uint32 => 4,
            Self::Int32 => 4,
            Self::Uint64 => 8,
            Self::Int64 => 8,
            Self::Bool => 1,
            Self::Unkown => 0,
        }
    }
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub size: usize,
    pub r#type: VariableType,
    pub offset: usize,
}

impl Variable {
    pub fn new() -> Self {
        Variable::default()
    }
}

impl Default for Variable {
    fn default() -> Self {
        Variable {
            name: String::new(),
            size: 0,
            r#type: VariableType::Unkown,
            offset: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct LocalVariables<'a> {
    pub variables: Vec<Variable>,
    pub previous: Option<&'a LocalVariables<'a>>,
}

impl<'a> LocalVariables<'a> {
    pub fn new() -> Self {
        LocalVariables::default()
    }
    /**
     * Find variable in local variables
     *
     * For example:
     * ```
     * println!("{:?}", variables.lookup("i"));
     * ```
     * Output:
     * ```
     * Some(Variable { name: "i", size: 8, type: Uint64, offset: 0 })
     * ```
     */
    pub fn lookup(&self, id: &str) -> Option<&Variable> {
        for i in &self.variables {
            if i.name == id {
                return Some(i);
            }
        }
        if let Some(previous) = self.previous {
            return previous.lookup(id);
        }
        None
    }
    pub fn modify_offset(&mut self, offset: isize) {
        for i in 0..self.variables.len() {
            if offset > 0 {
                self.variables[i].offset += offset as usize;
            } else {
                let offset: usize = (-offset).try_into().unwrap();
                self.variables[i].offset -= offset;
            }
        }
    }
    pub fn push(&mut self, var: Variable) -> Result<(), String> {
        if self.lookup(&var.name).is_some() {
            return Err(format!("'{}' has already defined", &var.name));
        }
        self.variables.push(var);
        Ok(())
    }
}

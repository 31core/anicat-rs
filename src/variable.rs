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
        match &var_type[..] {
            "u8" => return Self::Uint8,
            "i8" => return Self::Int8,
            "u16" => return Self::Uint16,
            "i16" => return Self::Int16,
            "u32" => return Self::Uint32,
            "i32" => return Self::Int32,
            "u64" => return Self::Uint64,
            "i64" => return Self::Int64,
            "bool" => return Self::Bool,
            _ => return Self::Unkown,
        }
    }
    pub fn get_size(&self) -> usize {
        match self {
            Self::Uint8 => return 1,
            Self::Int8 => return 1,
            Self::Uint16 => return 2,
            Self::Int16 => return 2,
            Self::Uint32 => return 4,
            Self::Int32 => return 4,
            Self::Uint64 => return 8,
            Self::Int64 => return 8,
            Self::Bool => return 1,
            Self::Unkown => return 0,
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
        Variable {
            name: String::new(),
            size: 0,
            r#type: VariableType::Unkown,
            offset: 0,
        }
    }
}

#[derive(Debug)]
pub struct LocalVariables {
    pub variables: Vec<Variable>,
    pub previous: Option<&'static LocalVariables>,
}

impl LocalVariables {
    pub fn new() -> Self {
        LocalVariables {
            variables: Vec::new(),
            previous: None,
        }
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
        None
    }
    pub fn modify_offset(&mut self, offset: isize) {
        for i in 0..self.variables.len() {
            if offset > 0 {
                self.variables[i].offset += offset as usize;
            } else {
                let offset: usize = (offset * -1).try_into().unwrap();
                self.variables[i].offset -= offset;
            }
        }
    }
    pub fn push(&mut self, var: Variable) -> Result<(), String> {
        if let Some(_) = self.lookup(&var.name) {
            return Err(format!("'{}' has already defined", &var.name));
        }
        self.variables.push(var);
        Ok(())
    }
}

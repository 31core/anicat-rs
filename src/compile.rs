use super::ast::*;
use super::vm::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
enum VariableTypes {
    Uint8,
    Int8,
    Uint16,
    Int16,
    Uint32,
    Int32,
    Uint64,
    Int64,
    BOOL,
    Unkown,
}

impl VariableTypes {
    fn from_string(var_type: &str) -> Self {
        match &var_type[..] {
            "u8" => return Self::Uint8,
            "i8" => return Self::Int8,
            "u16" => return Self::Uint16,
            "i16" => return Self::Int16,
            "u32" => return Self::Uint32,
            "i32" => return Self::Int32,
            "u64" => return Self::Uint64,
            "i64" => return Self::Int64,
            "bool" => return Self::BOOL,
            _ => return Self::Unkown,
        }
    }
    fn get_size(&self) -> usize {
        match self {
            Self::Uint8 => return 1,
            Self::Int8 => return 1,
            Self::Uint16 => return 2,
            Self::Int16 => return 2,
            Self::Uint32 => return 4,
            Self::Int32 => return 4,
            Self::Uint64 => return 8,
            Self::Int64 => return 8,
            Self::BOOL => return 1,
            _ => return 0,
        }
    }
}

#[derive(Debug)]
struct Variable {
    name: String,
    size: usize,
    r#type: VariableTypes,
    offset: usize,
    previous: Option<Rc<RefCell<Variable>>>,
}

impl Variable {
    fn new() -> Self {
        Variable {
            name: String::new(),
            size: 0,
            r#type: VariableTypes::Unkown,
            offset: 0,
            previous: None,
        }
    }
    /**
     * Find variable in the linked list
     *
     * For example:
     * ```
     * println!("{:?}", Variable::lookup(variable, "b"));
     * ```
     * Output:
     * ```
     * Some(RefCell { value: Variable { name: "b", size: 2, type: Uint16, offset: 12, previous: Some(RefCell { value: Variable { name: "a", size: 1, type: Uint8, offset: 14, previous: None } }) } })
     * ```
     */
    #[allow(dead_code)]
    fn lookup(var: Option<Rc<RefCell<Variable>>>, id: &str) -> Option<Rc<RefCell<Variable>>> {
        let mut var = var;
        loop {
            match var {
                Some(v) => {
                    if v.borrow().name == id {
                        return Some(Rc::clone(&v));
                    }
                    var = v.borrow().previous.clone();
                }
                None => return None,
            }
        }
    }
}

pub fn compile(byte_code: &mut Vec<u8>, ast: &AstNode) -> Result<(), ()> {
    let mut variable: Option<Rc<RefCell<Variable>>> = None;
    for node in &ast.nodes {
        if node.borrow().r#type == AST_TYPE_VAR_DECLARE {
            /* sub sp, u8: [var size] */
            byte_code.push(VM_OP_SUB);
            byte_code.push(VM_REG_SP);
            byte_code.push(VM_TYPE_VAL8);

            let var = Rc::new(RefCell::new(Variable::new()));
            var.borrow_mut().name = node.borrow().nodes[0].borrow().data.clone();
            var.borrow_mut().r#type =
                VariableTypes::from_string(&node.borrow().nodes[1].borrow().data);
            {
                let size = var.borrow().r#type.get_size();
                var.borrow_mut().size = size;
                byte_code.push(size as u8);
            }
            match &variable {
                Some(previous_var) => {
                    var.borrow_mut().previous = Some(Rc::clone(previous_var));
                    let mut this_var = Some(Rc::clone(previous_var));
                    variable = Some(Rc::clone(&var));

                    /* add offset of previous variables */
                    loop {
                        match this_var {
                            Some(v) => {
                                v.borrow_mut().offset += var.borrow().size;
                                this_var = v.borrow().previous.clone();
                            }
                            None => break,
                        }
                    }
                }
                None => variable = Some(var),
            }
        }
    }
    Ok(())
}

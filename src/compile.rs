use super::assembly::*;
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
pub struct Variable {
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
    fn modify_offset(
        var: Option<Rc<RefCell<Variable>>>,
        offset: isize,
    ) -> Option<Rc<RefCell<Variable>>> {
        let mut var = var;
        let mut last_var = None;
        loop {
            match var {
                Some(v) => {
                    if offset > 0 {
                        v.borrow_mut().offset += offset as usize;
                    } else {
                        let offset: usize = (offset * -1).try_into().unwrap();
                        /* out of life time */
                        if offset > v.borrow().offset {
                            last_var = v.borrow().previous.clone();
                        } else {
                            v.borrow_mut().offset -= offset;
                        }
                    }

                    var = v.borrow().previous.clone();
                }
                None => return last_var,
            }
        }
    }
}

/**
 * compile for operating tree  
 * **NOTE**: The result will be saved to C0
 */
fn compile_op(
    byte_code: &mut Vec<u8>,
    ast: &Rc<RefCell<AstNode>>,
    variable: &Option<Rc<RefCell<Variable>>>,
) {
    /* left value */
    if ast.borrow().node(0).r#type == AST_TYPE_VALUE {
        /*
        mov c0, val
        */
        let val: u64 = ast.borrow().node(0).data.parse().unwrap();
        byte_code.extend(assemblize(
            VM_OP_MOV,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Value64(val),
            ],
        ));
    }
    /* variable */
    else if ast.borrow().node(0).r#type == AST_TYPE_IDENTIFIER {
        let mut offset = 0;
        if let Some(var) = Variable::lookup(variable.clone(), &ast.borrow().node(0).data) {
            offset = var.borrow().offset;
        }

        /*
        mov ar, sp
        add ar, val16: offset
        load c0, ar
        */
        byte_code.extend(assemblize(
            VM_OP_MOV,
            &[
                AssemblyValue::Register(VM_REG_AR),
                AssemblyValue::Register(VM_REG_SP),
            ],
        ));

        byte_code.extend(assemblize(
            VM_OP_ADD,
            &[
                AssemblyValue::Register(VM_REG_AR),
                AssemblyValue::Value16(offset as u16),
            ],
        ));

        byte_code.extend(assemblize(
            VM_OP_LOAD64,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_AR),
            ],
        ));
    }
    /* operating result */
    else if ast.borrow().node(0).is_operator()
        || ast.borrow().node(0).r#type == AST_TYPE_SHL
        || ast.borrow().node(0).r#type == AST_TYPE_SHR
        || ast.borrow().node(0).r#type == AST_TYPE_MOD
    {
        compile_op(byte_code, &ast.borrow().nodes[0], variable);
    }

    /* right value */
    /* constant */
    if ast.borrow().node(1).r#type == AST_TYPE_VALUE {
        let val: u64 = ast.borrow().node(1).data.parse().unwrap();
        /* mov c1, val64: val */
        byte_code.extend(assemblize(
            VM_OP_MOV,
            &[
                AssemblyValue::Register(VM_REG_C1),
                AssemblyValue::Value64(val),
            ],
        ));
    }
    /* variable */
    else if ast.borrow().node(1).r#type == AST_TYPE_IDENTIFIER {
        let mut offset = 0;
        if let Some(var) = Variable::lookup(variable.clone(), &ast.borrow().node(0).data) {
            offset = var.borrow().offset;
        }

        /*
        mov ar, sp
        add ar, [offset]
        load ar, c3
        */
        byte_code.extend(assemblize(
            VM_OP_MOV,
            &[
                AssemblyValue::Register(VM_REG_AR),
                AssemblyValue::Register(VM_REG_SP),
            ],
        ));

        byte_code.extend(assemblize(
            VM_OP_ADD,
            &[
                AssemblyValue::Register(VM_REG_AR),
                AssemblyValue::Value16(offset as u16),
            ],
        ));

        byte_code.extend(assemblize(
            VM_OP_LOAD64,
            &[
                AssemblyValue::Register(VM_REG_C1),
                AssemblyValue::Register(VM_REG_AR),
            ],
        ));
    }
    /* operating result */
    else if ast.borrow().node(1).is_operator()
        || ast.borrow().node(1).r#type == AST_TYPE_SHL
        || ast.borrow().node(1).r#type == AST_TYPE_SHR
        || ast.borrow().node(1).r#type == AST_TYPE_MOD
    {
        /* push c0 */
        byte_code.extend(assemblize(
            VM_OP_PUSH,
            &[AssemblyValue::Register(VM_REG_C0)],
        ));
        compile_op(byte_code, &ast.borrow().nodes[1], variable);
        /* mov c1, c0 */
        byte_code.extend(assemblize(
            VM_OP_MOV,
            &[
                AssemblyValue::Register(VM_REG_C1),
                AssemblyValue::Register(VM_REG_C0),
            ],
        ));
        /* pop c0 */
        byte_code.extend(assemblize(VM_OP_POP, &[AssemblyValue::Register(VM_REG_C0)]));
    }
    /* [add/sub/mul/div] c0, c1 */
    match ast.borrow().r#type {
        AST_TYPE_ADD => byte_code.extend(assemblize(
            VM_OP_ADD,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_C1),
            ],
        )),
        AST_TYPE_SUB => byte_code.extend(assemblize(
            VM_OP_SUB,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_C1),
            ],
        )),
        AST_TYPE_MUL => byte_code.extend(assemblize(
            VM_OP_MUL,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_C1),
            ],
        )),
        AST_TYPE_DIV => byte_code.extend(assemblize(
            VM_OP_DIV,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_C1),
            ],
        )),
        AST_TYPE_MOD => byte_code.extend(assemblize(
            VM_OP_MOD,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_C1),
            ],
        )),
        AST_TYPE_SHL => byte_code.extend(assemblize(
            VM_OP_SHL,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_C1),
            ],
        )),
        AST_TYPE_SHR => byte_code.extend(assemblize(
            VM_OP_SHR,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_C1),
            ],
        )),
        _ => {}
    }
}

/**
 * Compile to byte code
 *
 * Example:
 * ```
 * let mut byte_code = Vec::new();
 * compile::compile(&mut byte_code, &ast, None);
 * ```
*/
pub fn compile(
    byte_code: &mut Vec<u8>,
    ast: &AstNode,
    variable: Option<Rc<RefCell<Variable>>>,
) -> Option<Rc<RefCell<Variable>>> {
    let mut variable = variable;
    let mut variable_size = 0;
    for node in &ast.nodes {
        if node.borrow().r#type == AST_TYPE_VAR_DECLARE {
            let new_var = Rc::new(RefCell::new(Variable::new()));
            new_var.borrow_mut().name = node.borrow().nodes[0].borrow().data.clone();
            new_var.borrow_mut().r#type =
                VariableTypes::from_string(&node.borrow().nodes[1].borrow().data);
            {
                let size = new_var.borrow().r#type.get_size();
                new_var.borrow_mut().size = size;
                /* sub sp, u16: [var size] */
                byte_code.extend(assemblize(
                    VM_OP_SUB,
                    &[
                        AssemblyValue::Register(VM_REG_SP),
                        AssemblyValue::Value16(size as u16),
                    ],
                ));
                variable_size += size;
            }
            if let Some(previous_var) = &variable {
                new_var.borrow_mut().previous = Some(Rc::clone(previous_var));

                /* add offset of previous variables */
                let size = new_var.borrow().size as isize;
                Variable::modify_offset(variable.clone(), size);
            }
            variable = Some(Rc::clone(&new_var));
        }
        if node.borrow().is_operator()
            || node.borrow().r#type == AST_TYPE_SHL
            || node.borrow().r#type == AST_TYPE_SHR
            || node.borrow().r#type == AST_TYPE_MOD
        {
            compile_op(byte_code, node, &variable);
        }
        if node.borrow().r#type == AST_TYPE_VAR_SET_VALUE {
            if node.borrow().node(1).r#type == AST_TYPE_VALUE {
                /*
                mov c0, [value]
                */
                byte_code.extend(assemblize(
                    VM_OP_MOV,
                    &[
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Value64(node.borrow().node(1).data.parse().unwrap()),
                    ],
                ));
            }
            if node.borrow().node(1).is_operator() {
                compile_op(byte_code, &node.borrow().nodes[1], &variable);
            }
            if node.borrow().node(1).r#type == AST_TYPE_IDENTIFIER {
                /*
                mov ar, sp
                add ar, [offest]
                load c0, ar
                */
                let mut offset = 0;
                let var = Variable::lookup(variable.clone(), &node.borrow().node(1).data);
                if let Some(var) = var {
                    offset = var.borrow().offset;
                }

                byte_code.extend(assemblize(
                    VM_OP_MOV,
                    &[
                        AssemblyValue::Register(VM_REG_AR),
                        AssemblyValue::Register(VM_REG_SP),
                    ],
                ));
                /* don't add `add ar, 0` */
                if offset > 0 {
                    byte_code.extend(assemblize(
                        VM_OP_ADD,
                        &[
                            AssemblyValue::Register(VM_REG_AR),
                            AssemblyValue::Value16(offset as u16),
                        ],
                    ));
                }
                byte_code.extend(assemblize(
                    VM_OP_LOAD64,
                    &[
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_AR),
                    ],
                ));
            }
            /*
            mov ar, sp
            add ar, [offest]
            store c0, ar
            */
            let mut offset = 0;
            let var = Variable::lookup(variable.clone(), &node.borrow().node(0).data);
            if let Some(var) = var {
                offset = var.borrow().offset;
            }
            byte_code.extend(assemblize(
                VM_OP_MOV,
                &[
                    AssemblyValue::Register(VM_REG_AR),
                    AssemblyValue::Register(VM_REG_SP),
                ],
            ));
            /* don't add `add ar, 0` */
            if offset > 0 {
                byte_code.extend(assemblize(
                    VM_OP_ADD,
                    &[
                        AssemblyValue::Register(VM_REG_AR),
                        AssemblyValue::Value16(offset as u16),
                    ],
                ));
            }
            byte_code.extend(assemblize(
                VM_OP_STORE64,
                &[
                    AssemblyValue::Register(VM_REG_C0),
                    AssemblyValue::Register(VM_REG_AR),
                ],
            ));
        }
    }

    if variable_size > 0 {
        /* add sp, val16: variable_size */
        byte_code.extend(assemblize(
            VM_OP_ADD,
            &[
                AssemblyValue::Register(VM_REG_SP),
                AssemblyValue::Value16(variable_size as u16),
            ],
        ));
    }
    variable = Variable::modify_offset(variable.clone(), variable_size as isize * -1);
    variable
}

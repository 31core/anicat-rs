use super::assembly::*;
use super::ast::*;
use super::symbol::Symbols;
use super::variable::*;
use super::vm::*;
use std::cell::RefCell;
use std::rc::Rc;

fn compile_if(
    byte_code: &mut Vec<u8>,
    ast: Rc<RefCell<AstNode>>,
    upper: Option<&LocalVariables>,
    symbols: &mut Symbols,
) -> Result<(), String> {
    let mut variables = LocalVariables::new();
    variables.previous = upper;
    compile_op(
        byte_code,
        Rc::clone(&ast.borrow().nodes[0].borrow().nodes[0]),
        &mut variables,
    )?;

    /*
    test? c0, val8: 1
    j?e out_of_code_block

    if_code_block:
    ...
    out_of_code_block:
    ...
    */
    let id = symbols.alloc(0);
    symbols.internal_ref(id, byte_code.len() as u64 + 3);
    byte_code.extend(assemblize(
        VM_OP_JNE,
        &[
            AssemblyValue::Register(VM_REG_C0),
            AssemblyValue::Value64(0),
        ],
    ));

    /* compile code block */
    compile(byte_code, &ast.borrow().nodes[1].borrow(), upper, symbols)?;

    symbols.modify_internal_sym(id, byte_code.len() as u64);
    Ok(())
}

/**
 * compile for operating tree  
 * **NOTE**: The result will be saved to C0
 */
fn compile_op(
    byte_code: &mut Vec<u8>,
    ast: Rc<RefCell<AstNode>>,
    variables: &mut LocalVariables,
) -> Result<(), String> {
    /* left value */
    if ast.borrow().node(0).r#type == AST_TYPE_VALUE {
        /*
        mov c0, val
        */
        let val = ast.borrow().node(0).get_value().unwrap();
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
        let offset;
        match variables.lookup(&ast.borrow().node(0).data) {
            Some(var) => offset = var.offset,
            None => return Err(format!("'{}' undefined.", &ast.borrow().node(0).data)),
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
    else if ast.borrow().node(0).is_operator() {
        compile_op(byte_code, Rc::clone(&ast.borrow().nodes[0]), variables)?;
    }

    /* right value */
    /* constant */
    if ast.borrow().node(1).r#type == AST_TYPE_VALUE {
        let val: u64 = ast.borrow().node(1).get_value().unwrap();
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
        let offset;
        match variables.lookup(&ast.borrow().node(1).data) {
            Some(var) => offset = var.offset,
            None => return Err(format!("'{}' undefined.", &ast.borrow().node(1).data)),
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
    else if ast.borrow().node(1).is_operator() {
        /* push c0 */
        byte_code.extend(assemblize(
            VM_OP_PUSH,
            &[AssemblyValue::Register(VM_REG_C0)],
        ));
        compile_op(byte_code, Rc::clone(&ast.borrow().nodes[1]), variables)?;
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
    let op;
    match ast.borrow().r#type {
        AST_TYPE_ADD => op = VM_OP_ADD,
        AST_TYPE_SUB => op = VM_OP_SUB,
        AST_TYPE_MUL => op = VM_OP_MUL,
        AST_TYPE_DIV => op = VM_OP_DIV,
        AST_TYPE_AND => op = VM_OP_AND,
        AST_TYPE_OR => op = VM_OP_OR,
        AST_TYPE_LOGIC_AND => op = VM_OP_AND,
        AST_TYPE_LOGIC_OR => op = VM_OP_OR,
        AST_TYPE_MOD => op = VM_OP_MOD,
        AST_TYPE_SHL => op = VM_OP_SHL,
        AST_TYPE_SHR => op = VM_OP_SHR,
        AST_TYPE_EQU => {
            byte_code.extend(assemblize(
                VM_OP_TESTEQ,
                &[
                    AssemblyValue::Register(VM_REG_C0),
                    AssemblyValue::Register(VM_REG_C0),
                    AssemblyValue::Register(VM_REG_C1),
                ],
            ));
            return Ok(());
        }
        AST_TYPE_NEQU => {
            byte_code.extend(assemblize(
                VM_OP_TESTNEQ,
                &[
                    AssemblyValue::Register(VM_REG_C0),
                    AssemblyValue::Register(VM_REG_C0),
                    AssemblyValue::Register(VM_REG_C1),
                ],
            ));
            return Ok(());
        }
        _ => return Ok(()), // this will be never executed
    }
    byte_code.extend(assemblize(
        op,
        &[
            AssemblyValue::Register(VM_REG_C0),
            AssemblyValue::Register(VM_REG_C1),
        ],
    ));
    Ok(())
}

fn compile_new_var(
    byte_code: &mut Vec<u8>,
    ast: Rc<RefCell<AstNode>>,
    variables: &mut LocalVariables,
) -> Result<(), String> {
    let mut new_var = Variable::new();
    new_var.name = ast.borrow().nodes[0].borrow().data.clone();
    new_var.r#type = VariableType::from_string(&ast.borrow().nodes[1].borrow().data);
    {
        let size = new_var.r#type.get_size();
        new_var.size = size;
        /* sub sp, u16: [var size] */
        byte_code.extend(assemblize(
            VM_OP_SUB,
            &[
                AssemblyValue::Register(VM_REG_SP),
                AssemblyValue::Value16(size as u16),
            ],
        ));
    }
    let size = new_var.size as isize;
    variables.modify_offset(size);
    variables.push(new_var)?;
    Ok(())
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
    upper: Option<&LocalVariables>,
    symbols: &mut Symbols,
) -> Result<(), String> {
    let mut variables = LocalVariables::new();
    variables.previous = upper;
    for node in &ast.nodes {
        if node.borrow().r#type == AST_TYPE_VAR_DECLARE {
            compile_new_var(byte_code, Rc::clone(node), &mut variables)?;
        }
        if node.borrow().is_operator() {
            compile_op(byte_code, Rc::clone(node), &mut variables)?;
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
                        AssemblyValue::Value64(node.borrow().node(1).get_value().unwrap()),
                    ],
                ));
            }
            if node.borrow().node(1).is_operator() {
                compile_op(
                    byte_code,
                    Rc::clone(&node.borrow().nodes[1]),
                    &mut variables,
                )
                .unwrap();
            }
            if node.borrow().node(1).r#type == AST_TYPE_IDENTIFIER {
                /*
                mov ar, sp
                add ar, [offest]
                load c0, ar
                */
                let mut offset = 0;
                let var = variables.lookup(&node.borrow().node(1).data);
                if let Some(var) = var {
                    offset = var.offset;
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
            let offset;
            match variables.lookup(&node.borrow().node(0).data) {
                Some(var) => offset = var.offset,
                None => return Err(format!("'{}' undefined", &node.borrow().node(0).data)),
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
        if node.borrow().r#type == AST_TYPE_RETURN {
            byte_code.extend(assemblize(VM_OP_RET, &[]));
            /* return a constant */
            if node.borrow().node(0).r#type == AST_TYPE_VALUE {
                let val = node.borrow().node(0).get_value().unwrap();
                byte_code.extend(assemblize(
                    VM_OP_MOD,
                    &[
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Value64(val),
                    ],
                ));
            }
        }
        if node.borrow().r#type == AST_TYPE_IF {
            compile_if(byte_code, Rc::clone(node), Some(&variables), symbols)?;
        }
    }

    let mut variable_size = 0;
    for i in &variables.variables {
        variable_size += i.size;
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
    Ok(())
}

use crate::assembly::*;
use crate::ast::*;
use crate::function::*;
use crate::symbol::Symbols;
use crate::variable::*;
use crate::vm::*;

pub const NORMAL_BASE_ADDR: u64 = 0;

#[derive(Default, Debug)]
pub struct Compiler {
    pub functions: Functions,
}

impl Compiler {
    pub fn compile(
        &mut self,
        ast: &AstNode,
        upper: Option<&LocalVariables>,
        symbols: &mut Symbols,
        base_addr: u64,
    ) -> Result<Vec<u8>, String> {
        self.compile_block(ast, upper, symbols, base_addr)
    }
    /**
     * Compile AST to byte code
     *
     * Example:
     * ```
     * let byte_code = compile::compile(&ast, None).unwrap();
     * ```
     */
    fn compile_block(
        &mut self,
        ast: &AstNode,
        upper: Option<&LocalVariables>,
        symbols: &mut Symbols,
        base_addr: u64,
    ) -> Result<Vec<u8>, String> {
        let mut byte_code = Vec::new();
        let mut variables = LocalVariables::new();
        variables.previous = upper;
        for node in &ast.nodes {
            if node.borrow().r#type == AST_TYPE_VAR_DECLARE {
                byte_code.extend(self.compile_new_var(&node.borrow(), &mut variables)?);
            }
            if node.borrow().is_operator() {
                byte_code.extend(self.compile_op(&node.borrow(), &mut variables)?);
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
                            AssemblyValue::Value64(node.borrow().node(1).get_value()?),
                        ],
                    ));
                }
                if node.borrow().node(1).is_operator() {
                    byte_code.extend(self.compile_op(&node.borrow().node(1), &mut variables)?);
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
                let offset = match variables.lookup(&node.borrow().node(0).data) {
                    Some(var) => var.offset,
                    None => return Err(format!("'{}' undefined", &node.borrow().node(0).data)),
                };
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
                /* return a constant */
                if node.borrow().node(0).r#type == AST_TYPE_VALUE {
                    let val = node.borrow().node(0).get_value()?;
                    byte_code.extend(assemblize(
                        VM_OP_MOV,
                        &[
                            AssemblyValue::Register(VM_REG_C0),
                            AssemblyValue::Value64(val),
                        ],
                    ));
                }
                byte_code.extend(assemblize(VM_OP_RET, &[]));
            }
            if node.borrow().r#type == AST_TYPE_IF {
                byte_code.extend(self.compile_if(
                    &node.borrow(),
                    Some(&variables),
                    symbols,
                    base_addr + byte_code.len() as u64,
                )?);
            }
            if node.borrow().r#type == AST_TYPE_WHILE {
                byte_code.extend(self.compile_while(
                    &node.borrow(),
                    Some(&variables),
                    symbols,
                    base_addr + byte_code.len() as u64,
                )?);
            }
            if node.borrow().r#type == AST_TYPE_FUNC_DEF {
                byte_code.extend(self.compile_func_def(
                    &node.borrow(),
                    symbols,
                    base_addr + byte_code.len() as u64,
                )?);
            }
            /* call a function */
            if node.borrow().r#type == AST_TYPE_FUNC_CALL {
                let func_name = node.borrow().node(0).data.clone();
                let addr = symbols.lookup(&func_name).unwrap();

                /* push args into stack */
                for arg in &node.borrow().get_params().unwrap().nodes {
                    byte_code.extend(assemblize(
                        VM_OP_MOV,
                        &[
                            AssemblyValue::Register(VM_REG_C0),
                            AssemblyValue::Value64(arg.borrow().data.parse().unwrap()),
                        ],
                    ));
                    byte_code.extend(assemblize(
                        VM_OP_PUSH,
                        &[AssemblyValue::Register(VM_REG_C0)],
                    ));
                }
                byte_code.extend(assemblize(VM_OP_CALL, &[AssemblyValue::Value64(addr)]));

                /* release args */
                byte_code.extend(assemblize(
                    VM_OP_ADD,
                    &[
                        AssemblyValue::Register(VM_REG_SP),
                        AssemblyValue::Value8(
                            (VM_USIZE * self.functions.lookup(&func_name).unwrap().params.len())
                                as u8,
                        ),
                    ],
                ));
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
        Ok(byte_code)
    }
    /** compile for func declaration */
    fn compile_func_def(
        &mut self,
        ast: &AstNode,
        symbols: &mut Symbols,
        base_addr: u64,
    ) -> Result<Vec<u8>, String> {
        let mut byte_code = Vec::new();
        let func_name = ast.node(0).data.clone();
        let mut local_vars = LocalVariables::new();
        symbols.add_external_symbol(&func_name, base_addr + byte_code.len() as u64)?;

        let mut func = Function::default();

        /* get params */
        for i in &ast.node(1).nodes {
            let mut new_var = Variable::new();
            new_var.name.clone_from(&i.borrow().node(0).data);
            new_var.r#type = VariableType::from_string(&i.borrow().node(1).data);
            func.params.push(new_var.r#type.clone());
            local_vars.push(new_var)?;
        }
        self.functions.add(&func_name, &func.params);
        local_vars.modify_offset(VM_USIZE as isize);

        /* compile code block */
        byte_code.extend(self.compile_block(
            &ast.get_code_block().unwrap(),
            Some(&local_vars),
            symbols,
            base_addr + byte_code.len() as u64,
        )?);
        Ok(byte_code)
    }

    /** compile for func declaration */
    fn compile_while(
        &mut self,
        ast: &AstNode,
        upper: Option<&LocalVariables>,
        symbols: &mut Symbols,
        base_addr: u64,
    ) -> Result<Vec<u8>, String> {
        let mut byte_code = Vec::new();
        let mut variables = LocalVariables::new();
        variables.previous = upper;

        let start_id = symbols.alloc_internal_symbol(base_addr);
        byte_code.extend(self.compile_op(&ast.node(0).node(0), &mut variables)?);

        /*
        start:
        test? c0, val8: 1
        jne out_of_code_block

        if_code_block:
        ...
        jmp start
        out_of_code_block:
        ...
        */
        let out_of_code_block_id = symbols.alloc_internal_symbol(0);
        symbols.internal_reference(out_of_code_block_id, base_addr + byte_code.len() as u64 + 3);
        byte_code.extend(assemblize(
            VM_OP_JNE,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Value64(0),
            ],
        ));

        /* compile code block */
        byte_code.extend(self.compile_block(
            &ast.get_code_block().unwrap(),
            upper,
            symbols,
            base_addr + byte_code.len() as u64,
        )?);
        symbols.internal_reference(start_id, base_addr + byte_code.len() as u64 + 2);
        byte_code.extend(assemblize(VM_OP_JMP, &[AssemblyValue::Value64(0)]));

        symbols.modify_internal_sym(out_of_code_block_id, base_addr + byte_code.len() as u64);
        Ok(byte_code)
    }

    /** compile for if compression */
    fn compile_if(
        &mut self,
        ast: &AstNode,
        upper: Option<&LocalVariables>,
        symbols: &mut Symbols,
        base_addr: u64,
    ) -> Result<Vec<u8>, String> {
        let mut byte_code = Vec::new();
        let mut variables = LocalVariables::new();
        variables.previous = upper;
        byte_code.extend(self.compile_op(&ast.node(0).node(0), &mut variables)?);

        /*
        test? c0, val8: 1
        j?e out_of_code_block

        if_code_block:
        ...
        out_of_code_block:
        ...
        */
        let out_of_code_block_id = symbols.alloc_internal_symbol(0);
        symbols.internal_reference(out_of_code_block_id, base_addr + byte_code.len() as u64 + 3);
        byte_code.extend(assemblize(
            VM_OP_JNE,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Value64(0),
            ],
        ));

        /* compile code block */
        byte_code.extend(self.compile_block(
            &ast.get_code_block().unwrap(),
            upper,
            symbols,
            base_addr + byte_code.len() as u64,
        )?);

        symbols.modify_internal_sym(out_of_code_block_id, base_addr + byte_code.len() as u64);
        Ok(byte_code)
    }

    /**
     * compile for operating tree  
     * **NOTE**: The result will be saved to C0
     */
    fn compile_op(&self, ast: &AstNode, variables: &mut LocalVariables) -> Result<Vec<u8>, String> {
        let mut byte_code = Vec::new();
        /* left value */
        if ast.node(0).r#type == AST_TYPE_VALUE {
            /*
            mov c0, val
            */
            let val = ast.node(0).get_value()?;
            byte_code.extend(assemblize(
                VM_OP_MOV,
                &[
                    AssemblyValue::Register(VM_REG_C0),
                    AssemblyValue::Value64(val),
                ],
            ));
        }
        /* variable */
        else if ast.node(0).r#type == AST_TYPE_IDENTIFIER {
            let offset = match variables.lookup(&ast.node(0).data) {
                Some(var) => var.offset,
                None => return Err(format!("'{}' undefined.", &ast.node(0).data)),
            };

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
        else if ast.node(0).is_operator() {
            byte_code.extend(self.compile_op(&ast.node(0), variables)?);
        }

        /* right value */
        /* constant */
        if ast.node(1).r#type == AST_TYPE_VALUE {
            let val: u64 = ast.node(1).get_value()?;
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
        else if ast.node(1).r#type == AST_TYPE_IDENTIFIER {
            let offset = match variables.lookup(&ast.node(1).data) {
                Some(var) => var.offset,
                None => return Err(format!("'{}' undefined.", &ast.node(1).data)),
            };

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
        else if ast.node(1).is_operator() {
            /* push c0 */
            byte_code.extend(assemblize(
                VM_OP_PUSH,
                &[AssemblyValue::Register(VM_REG_C0)],
            ));
            byte_code.extend(self.compile_op(&ast.node(1), variables)?);
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
        let op = match ast.r#type {
            AST_TYPE_ADD => VM_OP_ADD,
            AST_TYPE_SUB => VM_OP_SUB,
            AST_TYPE_MUL => VM_OP_MUL,
            AST_TYPE_DIV => VM_OP_DIV,
            AST_TYPE_AND => VM_OP_AND,
            AST_TYPE_OR => VM_OP_OR,
            AST_TYPE_XOR => VM_OP_XOR,
            AST_TYPE_LOGIC_AND => VM_OP_AND,
            AST_TYPE_LOGIC_OR => VM_OP_OR,
            AST_TYPE_MOD => VM_OP_MOD,
            AST_TYPE_SHL => VM_OP_SHL,
            AST_TYPE_SHR => VM_OP_SHR,
            AST_TYPE_EQU => {
                byte_code.extend(assemblize(
                    VM_OP_TESTEQ,
                    &[
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C1),
                    ],
                ));
                return Ok(byte_code);
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
                return Ok(byte_code);
            }
            AST_TYPE_GT => {
                byte_code.extend(assemblize(
                    VM_OP_TESTGT,
                    &[
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C1),
                    ],
                ));
                return Ok(byte_code);
            }
            AST_TYPE_LT => {
                byte_code.extend(assemblize(
                    VM_OP_TESTLT,
                    &[
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C1),
                    ],
                ));
                return Ok(byte_code);
            }
            AST_TYPE_GE => {
                byte_code.extend(assemblize(
                    VM_OP_TESTLE,
                    &[
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C1),
                    ],
                ));
                return Ok(byte_code);
            }
            AST_TYPE_LE => {
                byte_code.extend(assemblize(
                    VM_OP_TESTLE,
                    &[
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C0),
                        AssemblyValue::Register(VM_REG_C1),
                    ],
                ));
                return Ok(byte_code);
            }
            _ => return Ok(byte_code), // this will be never executed
        };
        byte_code.extend(assemblize(
            op,
            &[
                AssemblyValue::Register(VM_REG_C0),
                AssemblyValue::Register(VM_REG_C1),
            ],
        ));
        Ok(byte_code)
    }

    /** compile for variable declaration */
    fn compile_new_var(
        &self,
        ast: &AstNode,
        variables: &mut LocalVariables,
    ) -> Result<Vec<u8>, String> {
        let mut byte_code = Vec::new();
        let mut new_var = Variable::new();
        new_var.name.clone_from(&ast.node(0).data.clone());
        new_var.r#type = VariableType::from_string(&ast.node(1).data);
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
        Ok(byte_code)
    }
}

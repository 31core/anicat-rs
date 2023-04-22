use super::ast::AstNode;
use super::token::Token;
use super::vm::VM;
use std::fmt::Debug;

const AST_TYPES: [&str; 38] = [
    "AST_TYPE_UNDEFINED",
    "AST_TYPE_PROGRAM",
    "AST_TYPE_IDENTIFIER",
    "AST_TYPE_VAR_DECLARE",
    "AST_TYPE_VAR_TYPE",
    "AST_TYPE_VAR_SET_VALUE",
    "AST_TYPE_FUNC_DEF",
    "AST_TYPE_FUNC_CALL",
    "AST_TYPE_CODE_BLOCK",
    "AST_TYPE_PARAMS",
    "AST_TYPE_IF",
    "AST_TYPE_ELIF",
    "AST_TYPE_ELSE",
    "AST_TYPE_FOR",
    "AST_TYPE_WHILE",
    "AST_TYPE_ADD",
    "AST_TYPE_SUB",
    "AST_TYPE_MUL",
    "AST_TYPE_DIV",
    "AST_TYPE_MOD",
    "AST_TYPE_GT",
    "AST_TYPE_LT",
    "AST_TYPE_GE",
    "AST_TYPE_LE",
    "AST_TYPE_EQU",
    "AST_TYPE_NEQU",
    "AST_TYPE_SHL",
    "AST_TYPE_SHR",
    "AST_TYPE_AND",
    "AST_TYPE_OR",
    "AST_TYPE_LOGIC_AND",
    "AST_TYPE_LOGIC_OR",
    "AST_TYPE_VALUE",
    "AST_TYPE_BREAK",
    "AST_TYPE_CONTINUE",
    "AST_TYPE_RETURN",
    "AST_TYPE_INDEX",
    "AST_TYPE_CHILD",
];

impl Debug for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", AST_TYPES[self.r#type as usize], self.data)?;
        Ok(())
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{:?}", self.name, self.r#type)?;
        Ok(())
    }
}

impl Debug for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "C0: 0x{:08X}\n", self.c0)?;
        write!(f, "C1: 0x{:08X}\n", self.c1)?;
        write!(f, "C2: 0x{:08X}\n", self.c2)?;
        write!(f, "C3: 0x{:08X}\n", self.c3)?;
        write!(f, "SP: 0x{:08X}\n", self.sp)?;
        write!(f, "IP: 0x{:08X}\n", self.ip)?;
        write!(f, "AR: 0x{:08X}", self.ar)?;
        Ok(())
    }
}

fn _print_ast(node: &AstNode, re: usize) {
    fn print_node(node: &AstNode, tab: usize) {
        for _ in 0..tab {
            print!("\t");
        }
        println!("{node:?}");
    }
    print_node(node, re);
    for i in &node.nodes {
        _print_ast(&(i.borrow()), re + 1);
    }
}
/// print AST
pub fn print_ast(node: &AstNode) {
    _print_ast(node, 0);
}

/// print tokens
pub fn print_token(tokens: &[Token]) {
    for i in tokens {
        println!("{i:?}");
    }
}

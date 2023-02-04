use super::ast::AstNode;
use super::token::Token;
use super::vm::VM;
use std::fmt::Debug;

const AST_TYPES: [&str; 32] = [
    "AST_TYPE_UNDEFINED",
    "AST_TYPE_PROGRAM",
    "AST_TYPE_IDENTIFIER",
    "AST_TYPE_VAR_DECLARE",
    "AST_TYPE_VAR_TYPE",
    "AST_TYPE_VAR_SET_VALUE",
    "AST_TYPE_VAR_GET_VALUE",
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
    "AST_TYPE_GT",
    "AST_TYPE_LT",
    "AST_TYPE_GE",
    "AST_TYPE_LE",
    "AST_TYPE_EQU",
    "AST_TYPE_AND",
    "AST_TYPE_OR",
    "AST_TYPE_VALUE",
    "AST_TYPE_BREAK",
    "AST_TYPE_CONTINUE",
    "AST_TYPE_RETURN",
    "AST_TYPE_INDEX",
];

const TOKEN_TYPES: [&str; 27] = [
    "TOKEN_TYPE_UNKOWN",
    "TOKEN_TYPE_NAME",
    "TOKEN_TYPE_KEYWORD",
    "TOKEN_TYPE_EQU",
    "TOKEN_TYPE_EXPLAIN",
    "TOKEN_TYPE_LS_BKT",
    "TOKEN_TYPE_LM_BKT",
    "TOKEN_TYPE_LL_BKT",
    "TOKEN_TYPE_RS_BKT",
    "TOKEN_TYPE_RM_BKT",
    "TOKEN_TYPE_RL_BKT",
    "TOKEN_TYPE_ADD",
    "TOKEN_TYPE_SUB",
    "TOKEN_TYPE_MUL",
    "TOKEN_TYPE_DIV",
    "TOKEN_TYPE_GT",
    "TOKEN_TYPE_LT",
    "TOKEN_TYPE_ISEQU",
    "TOKEN_TYPE_GE",
    "TOKEN_TYPE_LE",
    "TOKEN_TYPE_NUMBER",
    "TOKEN_TYPE_SPLIT",
    "TOKEN_TYPE_STRING",
    "TOKEN_TYPE_AND",
    "TOKEN_TYPE_OR",
    "TOKEN_TYPE_LOGIC_AND",
    "TOKEN_TYPE_LOGIC_OR",
];

impl Debug for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", AST_TYPES[self.r#type as usize], self.data)?;
        Ok(())
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{}", self.name, TOKEN_TYPES[self.r#type as usize])?;
        Ok(())
    }
}

impl Debug for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "C0: 0x{:08X}\n", self.c0)?;
        write!(f, "SP: 0x{:08X}\n", self.sp)?;
        write!(f, "IP: 0x{:08X}", self.ip)?;
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

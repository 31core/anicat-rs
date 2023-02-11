pub mod ast;
pub mod compile;
pub mod debug;
pub mod token;
pub mod vm;
pub mod vram;

use ast::AstNode;
use vm::VM;

fn main() {
    //print!("\x1b[H\x1b[2J\x1b[3J");
    let code = std::fs::read_to_string("test.ac").unwrap();
    /* generate tokens */
    let tokens = token::generate_token(&code).unwrap();
    /* generate AST */
    let ast = AstNode::from_tokens(&mut tokens.into_iter());
    let mut byte_code = Vec::new();
    compile::compile(&mut byte_code, &ast).unwrap();
    byte_code.push(0x14);

    let mut vm = VM::new();
    vm.update_code(&byte_code);
    vm.run();
}

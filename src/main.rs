pub mod ast;
pub mod debug;
pub mod token;
pub mod vm;

//use ast::AstNode;
use vm::*;

fn main() {
    //print!("\x1b[H\x1b[2J\x1b[3J");
    let code = std::fs::read_to_string("test.ac").unwrap();
    let tokens = token::generate_token(&code);
    //debug::print_token(&tokens);
    //let ast = AstNode::from_tokens(&mut tokens.into_iter());
    //debug::print_ast(&ast);
    let mut vm = VM::new();
    vm.update_code(&[VM_OP_JMP, VM_TYPE_VAL8, 7,
        VM_OP_ADD, VM_REG_C0, VM_TYPE_VAL8, 1,
        VM_OP_HAL]);
    vm.run();
    println!("{:?}", vm);
}

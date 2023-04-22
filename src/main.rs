pub mod assembly;
pub mod ast;
pub mod compile;
pub mod debug;
pub mod symbol;
pub mod token;
pub mod variable;
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
    debug::print_ast(&ast);
    let mut byte_code = Vec::new();
    let mut symbols = symbol::Symbols::new();
    let result = compile::compile(&mut byte_code, &ast, None, &mut symbols);
    match result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    }
    byte_code.extend(assembly::assemblize(vm::VM_OP_HAL, &[]));
    symbols.link(&mut byte_code);

    std::fs::write("byte_code", &byte_code).unwrap();

    let mut vm = VM::new();
    vm.update_code(&byte_code);
    vm.run();
    println!("{vm:?}");
}

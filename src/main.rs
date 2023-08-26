mod assembly;
mod ast;
mod compile;
mod debug;
mod function;
mod symbol;
mod token;
mod variable;
mod vm;
mod vram;

use ast::AstNode;
use std::io::*;
use vm::VM;

fn main() -> std::io::Result<()> {
    //print!("\x1b[H\x1b[2J\x1b[3J");
    let code = std::fs::read_to_string("test.ac")?;
    /* generate tokens */
    let tokens = token::generate_token(&code).unwrap();
    /* generate AST */
    let ast = AstNode::from_tokens(&mut tokens.into_iter());
    //debug::print_ast(&ast);

    let mut symbols = symbol::Symbols::new();
    let compiler = compile::Compiler::default();
    let result = compiler.compile(&ast, None, &mut symbols, compile::NORMAL_BASE_ADDR);
    let mut byte_code = match result {
        Ok(byte_code) => byte_code,
        Err(e) => {
            eprintln!("{e}");
            return Err(Error::new(ErrorKind::Other, ""));
        }
    };
    byte_code.extend(assembly::assemblize(vm::VM_OP_HAL, &[]));
    symbols.link(&mut byte_code);

    std::fs::write("byte_code", &byte_code)?;

    let mut vm = VM::new();
    vm.update_code(&byte_code);
    vm.set_entry_point(symbols.lookup("main").unwrap());
    vm.run();
    println!("{vm:?}");
    Ok(())
}

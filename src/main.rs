pub mod ast;
pub mod debug;
pub mod token;

use ast::AstNode;

fn main() {
    print!("\x1b[H\x1b[2J\x1b[3J");
    let code = std::fs::read_to_string("test.ac").unwrap();
    let tokens = token::generate_token(&code);
    //debug::print_token(&tokens);
    let ast = AstNode::from_tokens(&mut tokens.into_iter());
    debug::print_ast(&ast);
}

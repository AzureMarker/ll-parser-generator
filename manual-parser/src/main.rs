use crate::parser::parse;
use logos::Lexer;
use std::io;
use std::io::Read;

mod ast;
mod lexer;
mod parser;

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let lexer = Lexer::new(input.as_str());
    let ast = parse(lexer).unwrap();

    println!("Ast: {:?}", ast);
}

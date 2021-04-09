#[macro_use]
extern crate lalrpop_util;

use crate::lexer::{StatefulLexer, Token};
use logos::Logos;
use std::io;
use std::io::Read;

#[cfg(test)]
#[macro_use]
mod grammar_tests;

mod ast;
mod lexer;
mod ll_table_gen;

lalrpop_mod!(
    #[allow(clippy::all)]
    parser
);

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let lexer = StatefulLexer::new(Token::lexer(&input));
    let ast = parser::GrammarParser::new().parse(lexer);

    println!("{:#?}", ast);
}

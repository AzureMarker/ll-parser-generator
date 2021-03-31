#[macro_use]
extern crate lalrpop_util;

use crate::lexer::{LalrpopLexerIter, Token};
use logos::Logos;
use std::io;

mod lexer;

lalrpop_mod!(
    #[allow(clippy::all)]
    parser
);

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let lexer = Token::lexer(&input);
    let lexer_iter = LalrpopLexerIter::new(lexer);
    let ast = parser::GrammarParser::new().parse(lexer_iter);

    println!("{:#?}", ast);
}

use crate::ast::AstTerm;
use logos::Lexer;
use std::io;
use std::io::{Read, Write};

mod ast;
mod lexer;

mod parser;

mod generated_parser {
    #![allow(clippy::all)]
    ll_parser_generator::ll_parser! {
        use crate::lexer::Token;
        use crate::ast::AstTerm;

        token Token {
            "(" = Token::LParen,
            ")" = Token::RParen,
            "NUMBER" = Token::Number,
        }

        grammar;

        pub Term: AstTerm = {
            <n:Number> => n,
            "(" <t:Term> ")" => AstTerm::Paren(Box::new(t)),
        };

        Number: AstTerm = "NUMBER" => AstTerm::Number;
    }
}

fn main() {
    let mut input = String::new();

    print!("Parser to use? [manual|generated] ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    let parse: Box<dyn Fn(_) -> AstTerm> = match input.trim() {
        "manual" => Box::new(|input| parser::parse(input).unwrap()),
        "generated" => Box::new(|input| generated_parser::parse(input).unwrap()),
        _ => {
            eprintln!("Invalid input");
            return;
        }
    };

    println!("Enter the input:");
    input.clear();
    io::stdin().read_to_string(&mut input).unwrap();

    let lexer = Lexer::new(input.as_str());
    let ast = parse(lexer);

    println!("Ast: {:?}", ast);
}

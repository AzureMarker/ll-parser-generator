#[macro_use]
extern crate lalrpop_util;

use crate::ast::AstGrammar;
use crate::lexer::{StatefulLexer, Token};
use crate::ll_table_gen::{compute_first, compute_follow, compute_nullable, compute_parse_table};
use lalrpop_util::ParseError;
use logos::Logos;
use std::io;
use std::io::Read;
use std::ops::Range;

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
    let ast = parse(&input).unwrap(); // TODO: return error

    let nullable = compute_nullable(&ast);
    let first = compute_first(&ast, &nullable);
    let follow = compute_follow(&ast, &nullable, &first);
    let parse_table = compute_parse_table(&ast, &nullable, &first, &follow);

    println!("Parsed grammar: {:#?}\n", ast);
    println!("Parse table: {:#?}", parse_table);
}

/// Parse a grammar. If there are errors during parsing, the error will be returned.
fn parse(grammar_str: &str) -> Result<AstGrammar, ParseError<usize, Token, Range<usize>>> {
    let lexer = StatefulLexer::new(Token::lexer(grammar_str));

    match parser::GrammarParser::new().parse(lexer) {
        Ok(parsed_grammar) => Ok(parsed_grammar),
        Err(ParseError::InvalidToken { location }) => {
            let (line, col) = index_to_line_col(grammar_str, location);
            eprintln!("Invalid token at line {}, column {}", line, col);
            Err(ParseError::InvalidToken { location })
        }
        Err(ParseError::UnrecognizedToken {
            token: (lspan, token, _rspan),
            expected,
        }) => {
            let (line, col) = index_to_line_col(grammar_str, lspan);
            eprintln!(
                "Unrecognized token '{:?}' at line {}, column {}, expected [{}]",
                token,
                line,
                col,
                expected.join(", ")
            );
            Err(ParseError::UnrecognizedToken {
                token: (lspan, token, _rspan),
                expected,
            })
        }
        Err(ParseError::UnrecognizedEOF { location, expected }) => {
            let (line, col) = index_to_line_col(grammar_str, location);
            eprintln!(
                "Unexpected EOF at line {}, column {}, expected [{}]",
                line,
                col,
                expected.join(", ")
            );
            Err(ParseError::UnrecognizedEOF { location, expected })
        }
        Err(ParseError::ExtraToken {
            token: (lspan, token, _rspan),
        }) => {
            let (line, col) = index_to_line_col(grammar_str, lspan);
            eprintln!(
                "Unexpected extra token '{:?}' at line {}, column {}",
                token, line, col
            );
            Err(ParseError::ExtraToken {
                token: (lspan, token, _rspan),
            })
        }
        Err(ParseError::User { error }) => {
            let token = &grammar_str[error.clone()];
            let (line, col) = index_to_line_col(grammar_str, error.start);
            eprintln!("Invalid token '{}' at line {}, column {}", token, line, col);
            Err(ParseError::User { error })
        }
    }
}

/// Convert an index of the file into a line and column index
fn index_to_line_col(file_str: &str, index: usize) -> (usize, usize) {
    let line = file_str
        .chars()
        .enumerate()
        .take_while(|(i, _)| *i != index)
        .filter(|(_, c)| *c == '\n')
        .count()
        + 1;
    let column = file_str[0..index]
        .chars()
        .rev()
        .take_while(|c| *c != '\n')
        .count()
        + 1;

    (line, column)
}
